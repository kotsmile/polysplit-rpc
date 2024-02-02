use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result as AnyhowResult;
use reqwest::Client;
use rocket::tokio::sync::RwLock;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::client::chainlist::ChainlistClient;
use crate::models::proxy::ProxyConfig;
use crate::repo::cache::CacheRepo;

pub enum EvmRpcError {
    Server,
    Client,
    Internal(String),
    Proxy(String),
    Timeout,
}

impl Display for EvmRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            EvmRpcError::Server => write!(f, "Server error"),
            EvmRpcError::Client => write!(f, "Client error"),
            EvmRpcError::Proxy(msg) => write!(f, "Proxy error: {msg}"),
            EvmRpcError::Internal(msg) => write!(f, "Internal error: {msg}"),
            EvmRpcError::Timeout => write!(f, "Timeout"),
        }
    }
}

impl Debug for EvmRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

#[derive(Deserialize)]
struct EvmRpcTestResponse {
    jsonrpc: String,
    id: u32,
    result: String,
}

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct Metric {
    pub response_time_ms: u128,
}

#[derive(Debug)]
pub enum RpcMetric {
    Ok(Metric),
    Error(String),
}

pub struct EvmRpcService {
    cache_repo: Arc<RwLock<CacheRepo>>,
    chainlist_client: Box<ChainlistClient>,
}

impl EvmRpcService {
    pub fn new(cache_repo: Arc<RwLock<CacheRepo>>, chainlist_client: Box<ChainlistClient>) -> Self {
        Self {
            cache_repo,
            chainlist_client,
        }
    }

    fn build_http_client(
        &self,
        proxy_config: Option<&ProxyConfig>,
        timeout: Duration,
    ) -> Result<Client, EvmRpcError> {
        match proxy_config {
            Some(proxy_config) => Client::builder()
                .timeout(timeout)
                .proxy(
                    proxy_config
                        .to_proxy()
                        .map_err(|err| EvmRpcError::Proxy(format!("{err}")))?,
                )
                .build()
                .map_err(|err| EvmRpcError::Internal(format!("{err}"))),
            None => Client::builder()
                .timeout(timeout)
                .build()
                .map_err(|err| EvmRpcError::Internal(format!("{err}"))),
        }
    }

    pub async fn rpc_request(
        &self,
        rpc: &str,
        proxy_config: Option<&ProxyConfig>,
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
                    return response
                        .json::<Value>()
                        .await
                        .map_err(|err| EvmRpcError::Internal(format!("parse error: {err}")));
                } else if response.status().is_client_error() {
                    return Err(EvmRpcError::Client);
                } else if response.status().is_server_error() {
                    return Err(EvmRpcError::Server);
                } else {
                    return Err(EvmRpcError::Internal(format!(
                        "unknown error: {}",
                        response.status()
                    )));
                }
            }
            Err(err) => {
                if err.is_timeout() {
                    return Err(EvmRpcError::Timeout);
                } else {
                    return Err(EvmRpcError::Internal(format!("unknow error: {err}")));
                }
            }
        }
    }

    pub async fn rpc_health_check(
        &self,
        chain_id: &str,
        rpc: &str,
        proxy_config: Option<&ProxyConfig>,
        timeout: Duration,
        request_tries: u32,
    ) -> RpcMetric {
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
                .rpc_request(rpc, proxy_config, &test_request, timeout)
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
            return RpcMetric::Error("Too many failed attempts".to_owned());
        }

        let response_time_ms = total_time / (request_tries - failed) as u128;
        return RpcMetric::Ok(Metric { response_time_ms });
    }

    pub async fn fetch_rpcs(&self) -> AnyhowResult<HashMap<String, Vec<String>>> {
        self.chainlist_client.fetch_rpcs().await
    }

    pub async fn set_rpcs_for_chain_id(&self, chain_id: &str, rpcs: Vec<(String, Metric)>) {
        self.cache_repo
            .write()
            .await
            .set_rpcs_for_chain_id(chain_id, rpcs)
    }

    pub async fn get_rpcs_for_chain_id(&self, chain_id: &str) -> Option<Vec<(String, Metric)>> {
        self.cache_repo.read().await.get_rpcs_for_chain_id(chain_id)
    }
}
