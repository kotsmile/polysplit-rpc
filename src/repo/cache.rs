use moka::sync::Cache;

use crate::{models::monitoring::Monitoring, services::evm_rpc::RpcMetrics};

pub struct CacheRepo {
    chain_id_to_rpcs_cache: Cache<String, Vec<(String, RpcMetrics)>>,
    monitoring: Monitoring,
}

impl CacheRepo {
    pub fn new() -> Self {
        Self {
            chain_id_to_rpcs_cache: Cache::builder().max_capacity(1024).build(),
            monitoring: Monitoring::new(),
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
}
