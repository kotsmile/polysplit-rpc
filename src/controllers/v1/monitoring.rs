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
    let total = monitoring_service.get_income_requets().await;
    let success = monitoring_service.get_succes_income_requets().await;
    Ok(ResponseData::build(MonitoringResponse {
        total,
        success,
        errors: total - success,
        success_rate: (success as f32 / total as f32) * 100 as f32,
    }))
}
