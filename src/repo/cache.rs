use moka::sync::Cache;

pub struct CacheRepo {
    chain_id_to_rpcs_cache: Cache<String, Vec<String>>,
    rpc_to_score_cache: Cache<String, i32>,
}

impl CacheRepo {
    pub fn new() -> Self {
        Self {
            chain_id_to_rpcs_cache: Cache::builder().max_capacity(1024).build(),
            rpc_to_score_cache: Cache::builder().max_capacity(1024).build(),
        }
    }

    pub fn get_rpcs_for_chain_id(&self, chain_id: &str) -> Option<Vec<String>> {
        self.chain_id_to_rpcs_cache.get(chain_id)
    }

    pub fn set_rpcs_for_chain_id(&mut self, chain_id: &str, rpcs: Vec<String>) {
        self.chain_id_to_rpcs_cache
            .insert(chain_id.to_string(), rpcs);
    }

    pub fn get_score_for_rpc(&self, rpc: &str) -> Option<i32> {
        self.rpc_to_score_cache.get(rpc)
    }

    pub fn set_score_for_rpc(&mut self, rpc: &str, score: i32) {
        self.rpc_to_score_cache.insert(rpc.to_string(), score);
    }
}
