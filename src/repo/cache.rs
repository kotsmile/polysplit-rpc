use moka::sync::Cache;

use crate::services::evm_rpc::RpcMetrics;

pub struct CacheRepo {
    chain_id_to_rpcs_cache: Cache<String, Vec<(String, RpcMetrics)>>,
    income_requests: u128,
    success_income_requests: u128,
}

impl CacheRepo {
    pub fn new() -> Self {
        Self {
            chain_id_to_rpcs_cache: Cache::builder().max_capacity(1024).build(),
            income_requests: 0,
            success_income_requests: 0,
        }
    }

    pub fn get_rpcs_for_chain_id(&self, chain_id: &str) -> Option<Vec<(String, RpcMetrics)>> {
        self.chain_id_to_rpcs_cache.get(chain_id)
    }

    pub fn set_rpcs_for_chain_id(&mut self, chain_id: &str, rpcs: Vec<(String, RpcMetrics)>) {
        self.chain_id_to_rpcs_cache
            .insert(chain_id.to_string(), rpcs);
    }

    pub fn get_income_requests(&self) -> u128 {
        self.income_requests
    }
    pub fn increment_income_requests(&mut self) {
        self.income_requests += 1;
    }

    pub fn get_success_income_requests(&self) -> u128 {
        self.success_income_requests
    }
    pub fn increment_success_income_requests(&mut self) {
        self.success_income_requests += 1;
    }
}
