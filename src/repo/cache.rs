use std::collections::{hash_map::Entry, HashMap};

use moka::sync::Cache;
use uuid::Uuid;

use crate::{
    models::{monitoring::Monitoring, Rpc},
    services::evm_rpc::RpcMetrics,
};

pub struct CacheRepo {
    chain_id_to_rpcs_cache: Cache<String, Vec<(String, RpcMetrics)>>,
    monitoring: Monitoring,
    user_to_monitoring: HashMap<Uuid, Monitoring>,
    api_key_to_rpcs: HashMap<String, Vec<Rpc>>,
}

impl CacheRepo {
    pub fn new() -> Self {
        Self {
            chain_id_to_rpcs_cache: Cache::builder().max_capacity(1024).build(),
            monitoring: Monitoring::new(),
            user_to_monitoring: HashMap::new(),
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

    pub fn get_user_monitoring(&self, user_id: &Uuid) -> Option<Monitoring> {
        self.user_to_monitoring.get(user_id).map(|v| v.clone())
    }

    pub fn get_user_monitoring_mut(&mut self, user_id: &Uuid) -> &mut Monitoring {
        match self.user_to_monitoring.entry(user_id.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Monitoring {
                income_requests: 0,
                success_income_requests: 0,
                error_income_requests: 0,
            }),
        }
    }

    pub fn get_map_key(api_key: &str, chain_id: &str) -> String {
        format!("{api_key}_{chain_id}")
    }

    pub fn get_rpcs_for_api_key(&self, api_key: &str, chain_id: &str) -> Option<Vec<Rpc>> {
        self.api_key_to_rpcs
            .get(&Self::get_map_key(api_key, chain_id))
            .map(|v| v.clone())
    }

    pub fn set_rpcs_for_api_key(&mut self, api_key: &str, chain_id: &str, rpcs: Vec<Rpc>) {
        self.api_key_to_rpcs
            .insert(Self::get_map_key(api_key, chain_id), rpcs);
    }
}
