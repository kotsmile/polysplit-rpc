use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context};
use reqwest::Client;
use rocket::async_trait;
use rocket::form::validate::Contains;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

use crate::client::chainlist::{ChainConfig, ChainlistClient};
use crate::models::proxy::ProxyConfig;
use crate::models::{Chain, NewRpc, Rpc, RpcVisibility};

#[derive(Debug, Error)]
pub enum EvmRpcError {
    #[error("server error")]
    Server,
    #[error("client error")]
    Client,
    #[error("internal error: {0}")]
    Internal(anyhow::Error),
    #[error("proxy error: {0}")]
    Proxy(anyhow::Error),
    #[error("rpc timeout")]
    Timeout,
}

#[derive(Deserialize)]
pub struct EvmRpcTestResponse {
    pub jsonrpc: String,
    pub id: u32,
    pub result: String,
}

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct RpcMetrics {
    pub response_time_ms: u128,
}

impl RpcMetrics {
    pub fn to_score(&self) -> f32 {
        1.0 / self.response_time_ms as f32
    }
}

#[async_trait]
pub trait EvmRpcStorage: Send + Sync + 'static {
    async fn get_chains(&self) -> anyhow::Result<Vec<Chain>>;
    async fn create_chains(&self, chains: &Vec<Chain>) -> anyhow::Result<()>;
    async fn get_public_rpcs_by_chain_id(&self, chain_id: &str) -> anyhow::Result<Vec<Rpc>>;
    async fn get_rpcs_by_chain_id(&self, chain_id: &str) -> anyhow::Result<Vec<Rpc>>;
    async fn create_rpcs(&self, new_rpcs: &Vec<NewRpc>) -> anyhow::Result<()>;
}

#[async_trait]
pub trait EvmRpcCache: Send + Sync + 'static {
    async fn get_rpcs_for_api_key(&self, api_key: &str, chain_id: &str) -> Option<Vec<String>>;
    async fn set_rpcs_for_api_key(&self, api_key: &str, chain_id: &str, rpcs: Vec<String>);
    async fn get_rpcs_for_chain_id(&self, chain_id: &str) -> Option<Vec<(String, RpcMetrics)>>;
    async fn set_rpcs_for_chain_id(&self, chain_id: &str, rpcs: Vec<(String, RpcMetrics)>);
}

pub struct EvmRpcService {
    chainlist_client: Box<ChainlistClient>,
    evm_cache: Arc<dyn EvmRpcCache>,
    evm_storage_repo: Arc<dyn EvmRpcStorage>,
}

impl EvmRpcService {
    pub fn new(
        chainlist_client: Box<ChainlistClient>,
        evm_cache: Arc<dyn EvmRpcCache>,
        evm_storage_repo: Arc<dyn EvmRpcStorage>,
    ) -> Self {
        Self {
            chainlist_client,
            evm_cache,
            evm_storage_repo,
        }
    }

    fn build_http_client(
        &self,
        proxy_config: Option<ProxyConfig>,
        timeout: Duration,
    ) -> Result<Client, EvmRpcError> {
        match proxy_config {
            Some(proxy_config) => Client::builder()
                .timeout(timeout)
                .proxy(proxy_config.to_proxy().map_err(EvmRpcError::Proxy)?)
                .build()
                .context("failed to build http client")
                .map_err(EvmRpcError::Internal),
            None => Client::builder()
                .timeout(timeout)
                .build()
                .context("failed to build http client")
                .map_err(EvmRpcError::Internal),
        }
    }

    pub async fn init_chains(&self, chain_ids: &Vec<String>) -> anyhow::Result<()> {
        let chains_configs = self
            .get_chains()
            .await
            .context("failed to get chains in evm_rpc service")?;

        let mut chains: Vec<Chain> = Vec::new();
        for chain in chains_configs {
            if !chain_ids.contains(&chain.chain_id) {
                continue;
            }

            chains.push(Chain {
                id: chain.chain_id,
                name: chain.name,
            })
        }

        self.evm_storage_repo
            .create_chains(&chains)
            .await
            .context("failed to create chains in storage repo")?;

        Ok(())
    }

    pub async fn update_public_rpcs(&self) -> anyhow::Result<()> {
        let chain_id_to_rpcs = self
            .chainlist_client
            .fetch_rpcs()
            .await
            .context("failed to fetch rpcs from chainlist client")?;

        let chains = self
            .evm_storage_repo
            .get_chains()
            .await
            .context("failed to get chains from storage repo")?;

        for chain in &chains {
            let Some(rpcs) = chain_id_to_rpcs.get(&chain.id) else {
                log::warn!("no rpcs was found for {chain_id}", chain_id = chain.id);
                continue;
            };

            self.evm_storage_repo
                .create_rpcs(
                    &rpcs
                        .iter()
                        .map(|val| NewRpc {
                            visibility: RpcVisibility::Public,
                            chain_id: chain.id.clone(),
                            url: val.to_string(),
                        })
                        .collect(),
                )
                .await
                .context(format!(
                    "failed to create rpcs for {chain_id}",
                    chain_id = chain.id
                ))?;
        }

        Ok(())
    }

    pub async fn get_chains(&self) -> anyhow::Result<Vec<ChainConfig>> {
        self.chainlist_client
            .fetch_chains()
            .await
            .context("failed to fetch chains from chainlist client")
    }

    pub async fn rpc_request(
        &self,
        rpc: &str,
        proxy_config: Option<ProxyConfig>,
        body: &Value,
        timeout: Duration,
    ) -> Result<Value, EvmRpcError> {
        let response = self
            .build_http_client(proxy_config, timeout)?
            .post(rpc)
            .json(body)
            .send()
            .await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    response
                        .json::<Value>()
                        .await
                        .context("failed to parse response")
                        .map_err(EvmRpcError::Internal)
                } else if response.status().is_client_error() {
                    Err(EvmRpcError::Client)
                } else if response.status().is_server_error() {
                    Err(EvmRpcError::Server)
                } else {
                    Err(EvmRpcError::Internal(anyhow!(
                        "unknown error: {}",
                        response.status()
                    )))
                }
            }
            Err(err) => {
                if err.is_timeout() {
                    Err(EvmRpcError::Timeout)
                } else {
                    Err(EvmRpcError::Internal(anyhow!("unknown error: {err}")))
                }
            }
        }
    }

    pub async fn rpc_health_check(
        &self,
        chain_id: &str,
        rpc: &str,
        proxy_config: Option<ProxyConfig>,
        timeout: Duration,
        request_tries: u32,
    ) -> anyhow::Result<RpcMetrics> {
        let test_request = json!({
            "method": "eth_chainId",
            "params": [],
            "id": 1,
            "jsonrpc": "2.0",
        });

        let mut total_time = 0;
        let mut failed = 0;

        for _ in 0..request_tries {
            let start = Instant::now();

            let response = self
                .rpc_request(rpc, proxy_config.clone(), &test_request, timeout)
                .await;

            let elapsed = start.elapsed();

            match response {
                Ok(value) => {
                    let result = serde_json::from_value::<EvmRpcTestResponse>(value)
                        .ok()
                        .and_then(|val| val.result.strip_prefix("0x").map(|v| v.to_owned()))
                        .and_then(|val| i64::from_str_radix(&val, 16).ok())
                        .map(|val| format!("{val}"));

                    let Some(real_chain_id) = result else {
                        failed += 1;
                        continue;
                    };

                    if real_chain_id != chain_id {
                        failed += 1;
                        continue;
                    }
                }
                Err(err) => {
                    failed += 1;
                    log::debug!("failed to check rpc {rpc}: {err}");
                    continue;
                }
            }

            total_time += elapsed.as_millis();
        }

        if failed > 0 {
            bail!("Too many failed attempts")
        }

        let response_time_ms = total_time / (request_tries - failed) as u128;
        return Ok(RpcMetrics { response_time_ms });
    }

    pub async fn get_public_rpcs_for_chain_id(&self, chain_id: &str) -> anyhow::Result<Vec<Rpc>> {
        self.evm_storage_repo
            .get_public_rpcs_by_chain_id(chain_id)
            .await
            .context("failed to get public rpcs for chain id from storage repo")
    }

    pub async fn fetch_rpcs(&self) -> anyhow::Result<HashMap<String, Vec<String>>> {
        self.chainlist_client.fetch_rpcs().await
    }

    pub async fn get_rpcs_for_chain_id(&self, chain_id: &str) -> anyhow::Result<Vec<Rpc>> {
        self.evm_storage_repo
            .get_rpcs_by_chain_id(chain_id)
            .await
            .context("failed to to get rpcs for chain id from storage repo")
    }

    pub async fn set_rpcs_for_chain_id_cache(
        &self,
        chain_id: &str,
        rpcs: Vec<(String, RpcMetrics)>,
    ) {
        self.evm_cache.set_rpcs_for_chain_id(chain_id, rpcs).await;
    }

    pub async fn get_rpcs_for_chain_id_cache(
        &self,
        chain_id: &str,
    ) -> Option<Vec<(String, RpcMetrics)>> {
        self.evm_cache.get_rpcs_for_chain_id(chain_id).await
    }

    pub async fn get_rpcs_for_api_key_cache(
        &self,
        api_key: &str,
        chain_id: &str,
    ) -> Option<Vec<String>> {
        self.evm_cache.get_rpcs_for_api_key(api_key, chain_id).await
    }

    pub async fn set_rpcs_for_api_key_cache(
        &self,
        api_key: &str,
        chain_id: &str,
        rpcs: Vec<String>,
    ) {
        if api_key == "" {
            return;
        }

        self.evm_cache
            .set_rpcs_for_api_key(api_key, chain_id, rpcs)
            .await;
    }
}
