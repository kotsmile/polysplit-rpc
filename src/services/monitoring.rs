use std::sync::Arc;

use rocket::tokio::sync::RwLock;

use crate::{models::monitoring::Monitoring, repo::cache::CacheRepo};

pub struct MonitoringService {
    cache_repo: Arc<RwLock<CacheRepo>>,
}

impl MonitoringService {
    pub fn new(cache_repo: Arc<RwLock<CacheRepo>>) -> Self {
        Self { cache_repo }
    }

    pub async fn get_monitoring(&self) -> Monitoring {
        self.cache_repo.read().await.get_monitoring().clone()
    }

    pub async fn inc_income_requests(&self) {
        self.cache_repo.write().await.update_monitoring(|m| {
            m.income_requests += 1;
        });
    }

    pub async fn inc_success_income_requests(&self) {
        self.cache_repo.write().await.update_monitoring(|m| {
            m.success_income_requests += 1;
        });
    }

    pub async fn inc_error_income_requests(&self) {
        self.cache_repo.write().await.update_monitoring(|m| {
            m.error_income_requests += 1;
        });
    }
}
