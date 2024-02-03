use std::sync::Arc;

use rocket::{get, State};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    services::monitoring::MonitoringService,
    util::controllers::{ResponseData, ResponseResultData},
};

#[derive(Debug, Serialize, JsonSchema)]
pub struct MonitoringResponse {
    total: u128,
    success: u128,
    errors: u128,
    success_rate: f32,
}

#[openapi(tag = "Monitoring")]
#[get("/v1/monitoring")]
pub async fn get_monitoring_v1(
    monitoring_service: &State<Arc<MonitoringService>>,
) -> ResponseResultData<MonitoringResponse> {
    let monitoring = monitoring_service.get_monitoring().await;
    Ok(ResponseData::build(MonitoringResponse {
        total: monitoring.income_requests,
        success: monitoring.success_income_requests,
        errors: monitoring.error_income_requests,
        success_rate: 100.0
            - (monitoring.error_income_requests as f32 / monitoring.income_requests as f32) * 100.0,
    }))
}
