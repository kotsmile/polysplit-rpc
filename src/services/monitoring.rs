use std::sync::Arc;

use rocket::async_trait;

use crate::models::monitoring::Monitoring;

pub struct MonitoringService {
    monitoring_cache: Arc<dyn MonitoringCache>,
}

#[async_trait]
pub trait MonitoringCache: Send + Sync + 'static {
    async fn get_monitoring(&self) -> Monitoring;
    async fn increment_income_requests(&self);
    async fn increment_success_income_requests(&self);
    async fn increment_error_income_requests(&self);
}

impl MonitoringService {
    pub fn new(monitoring_cache: Arc<dyn MonitoringCache>) -> Self {
        Self { monitoring_cache }
    }

    pub async fn get_monitoring(&self) -> Monitoring {
        self.monitoring_cache.get_monitoring().await
    }

    pub async fn inc_income_requests(&self) {
        self.monitoring_cache.increment_income_requests().await;
    }

    pub async fn inc_success_income_requests(&self) {
        self.monitoring_cache
            .increment_success_income_requests()
            .await;
    }

    pub async fn inc_error_income_requests(&self) {
        self.monitoring_cache
            .increment_error_income_requests()
            .await;
    }
}
