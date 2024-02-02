use std::sync::Arc;

use rocket::tokio::sync::RwLock;

use crate::repo::cache::CacheRepo;

pub struct MonitoringService {
    cache_repo: Arc<RwLock<CacheRepo>>,
}

impl MonitoringService {
    pub fn new(cache_repo: Arc<RwLock<CacheRepo>>) -> Self {
        Self { cache_repo }
    }

    pub async fn increment_income_requests(&self) {
        self.cache_repo.write().await.increment_income_requests();
    }

    pub async fn get_income_requets(&self) -> u128 {
        self.cache_repo.read().await.get_income_requests()
    }

    pub async fn increment_success_income_requests(&self) {
        self.cache_repo
            .write()
            .await
            .increment_success_income_requests();
    }

    pub async fn get_succes_income_requets(&self) -> u128 {
        self.cache_repo.read().await.get_success_income_requests()
    }
}
