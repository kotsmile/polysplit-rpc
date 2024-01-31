use anyhow::Result;

use crate::{models::proxy::ProxyConfig, traits::ProxyClient};

pub struct ProxyService {
    proxy_client: Box<dyn ProxyClient>,
    proxy_id: usize,
    proxies: Vec<ProxyConfig>,
}

impl ProxyService {
    pub fn new(proxy_client: Box<dyn ProxyClient>) -> Self {
        Self {
            proxy_client,
            proxy_id: 0,
            proxies: Vec::new(),
        }
    }

    pub fn get_proxy(&self) -> Option<&ProxyConfig> {
        self.proxies.get(self.proxy_id)
    }

    pub fn init_proxies(&mut self) -> Result<()> {}
}
