use std::collections::HashMap;

use moka::sync::Cache;

use crate::{models::monitoring::Monitoring, services::evm_rpc::RpcMetrics};

pub struct CacheRepo {
    chain_id_to_rpcs_cache: Cache<String, Vec<(String, RpcMetrics)>>,
    monitoring: Monitoring,
    api_key_to_rpcs: HashMap<String, HashMap<String, Vec<String>>>,
}

impl CacheRepo {
    pub fn new() -> Self {
        Self {
            chain_id_to_rpcs_cache: Cache::builder().max_capacity(1024).build(),
            monitoring: Monitoring::new(),
            api_key_to_rpcs: HashMap::new(),
        }
    }

    pub fn get_rpcs_for_chain_id(&self, chain_id: &str) -> Option<Vec<(String, RpcMetrics)>> {
        self.chain_id_to_rpcs_cache.get(chain_id)
    }

    pub fn set_rpcs_for_chain_id(&mut self, chain_id: &str, rpcs: Vec<(String, RpcMetrics)>) {
        self.chain_id_to_rpcs_cache
            .insert(chain_id.to_string(), rpcs);
    }

    pub fn get_monitoring(&self) -> Monitoring {
        self.monitoring
    }

    pub fn get_monitoring_mut(&mut self) -> &mut Monitoring {
        &mut self.monitoring
    }

    pub fn update_api_key(&mut self, old_api_key: &str, new_api_key: &str) {
        if let Some(value) = self.api_key_to_rpcs.remove(old_api_key) {
            self.api_key_to_rpcs.insert(new_api_key.to_string(), value);
        }
    }

    pub fn get_rpcs_for_api_key(&self, api_key: &str, chain_id: &str) -> Option<Vec<String>> {
        self.api_key_to_rpcs
            .get(api_key)?
            .get(chain_id)
            .map(|v| v.clone())
    }

    pub fn set_rpcs_for_api_key(&mut self, api_key: &str, chain_id: &str, rpcs: Vec<String>) {
        self.api_key_to_rpcs
            .entry(api_key.to_string())
            .or_insert(HashMap::new())
            .entry(chain_id.to_string())
            .or_insert(rpcs);
    }
}
