use anyhow::Result;
use async_trait::async_trait;

use crate::models::proxy::ProxyConfig;

#[async_trait]
pub trait ProxyClient: Send + Sync + 'static {
    async fn fetch_proxies(&self) -> Result<Vec<ProxyConfig>>;
    async fn check_proxy(&self, proxy_config: &ProxyConfig) -> Result<bool>;
}
