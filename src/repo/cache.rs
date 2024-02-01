use moka::sync::Cache;

use crate::services::evm_rpc::Metric;

pub struct CacheRepo {
    chain_id_to_rpcs_cache: Cache<String, Vec<(String, Metric)>>,
}

impl CacheRepo {
    pub fn new() -> Self {
        Self {
            chain_id_to_rpcs_cache: Cache::builder().max_capacity(1024).build(),
        }
    }

    pub fn get_rpcs_for_chain_id(&self, chain_id: &str) -> Option<Vec<(String, Metric)>> {
        self.chain_id_to_rpcs_cache.get(chain_id)
    }

    pub fn set_rpcs_for_chain_id(&mut self, chain_id: &str, rpcs: Vec<(String, Metric)>) {
        self.chain_id_to_rpcs_cache
            .insert(chain_id.to_string(), rpcs);
    }
}
