use std::collections::HashMap;

use moka::sync::Cache;
use rocket::async_trait;
use rocket::tokio::sync::RwLock;

use crate::{
    models::monitoring::Monitoring,
    services::{
        evm_rpc::{EvmRpcCache, RpcMetrics},
        group::GroupCache,
        monitoring::MonitoringCache,
    },
};

pub struct CacheRepo {
    chain_id_to_rpcs_cache: RwLock<Cache<String, Vec<(String, RpcMetrics)>>>,
    monitoring: RwLock<Monitoring>,
    api_key_to_rpcs: RwLock<HashMap<String, HashMap<String, Vec<String>>>>,
}

impl CacheRepo {
    pub fn new() -> Self {
        Self {
            chain_id_to_rpcs_cache: RwLock::new(Cache::builder().max_capacity(1024).build()),
            monitoring: RwLock::new(Monitoring::new()),
            api_key_to_rpcs: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl EvmRpcCache for CacheRepo {
    async fn get_rpcs_for_api_key(&self, api_key: &str, chain_id: &str) -> Option<Vec<String>> {
        let api_key_to_rpcs = self.api_key_to_rpcs.read().await;
        api_key_to_rpcs
            .get(api_key)?
            .get(chain_id)
            .map(|v| v.clone())
    }

    async fn set_rpcs_for_api_key(&self, api_key: &str, chain_id: &str, rpcs: Vec<String>) {
        let mut api_key_to_rpcs = self.api_key_to_rpcs.write().await;
        api_key_to_rpcs
            .entry(api_key.to_string())
            .or_insert(HashMap::new())
            .entry(chain_id.to_string())
            .or_insert(rpcs);
    }

    async fn get_rpcs_for_chain_id(&self, chain_id: &str) -> Option<Vec<(String, RpcMetrics)>> {
        self.chain_id_to_rpcs_cache.read().await.get(chain_id)
    }

    async fn set_rpcs_for_chain_id(&self, chain_id: &str, rpcs: Vec<(String, RpcMetrics)>) {
        self.chain_id_to_rpcs_cache
            .write()
            .await
            .insert(chain_id.to_string(), rpcs);
    }
}

#[async_trait]
impl GroupCache for CacheRepo {
    async fn update_api_key(&self, old_api_key: &str, new_api_key: &str) {
        let mut api_key_to_rpcs = self.api_key_to_rpcs.write().await;
        if let Some(value) = api_key_to_rpcs.remove(old_api_key) {
            api_key_to_rpcs.insert(new_api_key.to_string(), value);
        }
    }
}

#[async_trait]
impl MonitoringCache for CacheRepo {
    async fn get_monitoring(&self) -> Monitoring {
        self.monitoring.read().await.clone()
    }

    async fn increment_income_requests(&self) {
        self.monitoring.write().await.income_requests += 1;
    }

    async fn increment_success_income_requests(&self) {
        self.monitoring.write().await.success_income_requests += 1;
    }

    async fn increment_error_income_requests(&self) {
        self.monitoring.write().await.error_income_requests += 1;
    }
}
