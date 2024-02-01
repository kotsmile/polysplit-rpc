use std::sync::Arc;

use rocket::tokio::sync::RwLock;

use crate::services::proxy::ProxyService;

pub struct ProxyUpdaterLib {
    proxy_service: Arc<RwLock<ProxyService>>,
}

impl ProxyUpdaterLib {
    pub fn new(proxy_service: Arc<RwLock<ProxyService>>) -> Self {
        Self { proxy_service }
    }

    pub async fn proxy_updater_cron(&self) {
        let _ = self
            .proxy_service
            .write()
            .await
            .rotate_proxy()
            .await
            .map_err(|err| log::error!("failed to rotate proxy: {err}"));
    }
}
