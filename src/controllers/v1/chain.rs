use std::sync::Arc;

use anyhow::anyhow;
use rocket::{get, http::Status, post, serde::json::Json, tokio::sync::RwLock, State};
use rocket_governor::RocketGovernor;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

use crate::{
    middleware::RateLimitGuard,
    repo::config::ConfigRepo,
    services::{
        evm_rpc::{EvmRpcService, RpcMetrics},
        monitoring::MonitoringService,
        proxy::ProxyService,
    },
    util::controllers::{ResponseData, ResponseError, ResponseResult, ResponseResultData},
};

#[openapi(tag = "Chain")]
#[get("/v1/chains/supported")]
pub async fn get_chains(config_repo: &State<ConfigRepo>) -> ResponseResultData<Vec<String>> {
    Ok(ResponseData::build(config_repo.supported_chain_ids.clone()))
}

#[post("/v1/chain/<chain_id>", format = "json", data = "<rpc_call>")]
pub async fn post_chain(
    chain_id: &str,
    rpc_call: Json<Value>,
    evm_rpc_service: &State<Arc<EvmRpcService>>,
    proxy_service: &State<Arc<RwLock<ProxyService>>>,
    monitoring_service: &State<Arc<MonitoringService>>,
    config_repo: &State<ConfigRepo>,
    _limitguard: RocketGovernor<'_, RateLimitGuard>,
) -> ResponseResult<Value> {
    monitoring_service.inc_income_requests().await;

    if let None = config_repo
        .supported_chain_ids
        .iter()
        .position(|val| val == chain_id)
    {
        monitoring_service.inc_error_income_requests().await;
        return Err(ResponseError {
            status: Status::BadRequest,
            error: format!("chainId {chain_id} is not supported yet"),
            internal_error: Err(anyhow!("chainId {chain_id} is not supported")),
        });
    }

    let Some(rpcs) = evm_rpc_service.get_rpcs_for_chain_id_cache(chain_id).await else {
        monitoring_service.inc_error_income_requests().await;
        return Err(ResponseError {
            status: Status::InternalServerError,
            error: format!("No rpc provided for chainId {chain_id}"),
            internal_error: Err(anyhow!("failed to get rpcs for chainId {chain_id}")),
        });
    };

    let rpc_call = rpc_call.into_inner();
    let proxy_service = proxy_service.read().await;
    let proxy_config = proxy_service.get_proxy();
    for i in 1..3 {
        for rpc in &rpcs {
            let response = evm_rpc_service
                // TODO(@kotsmile): add proxy handling
                .rpc_request(
                    &rpc.0,
                    proxy_config,
                    &rpc_call,
                    config_repo.feed_max_timeout * i,
                )
                .await;

            match response {
                Ok(val) => {
                    log::info!("picked rpc: {}", rpc.0);
                    monitoring_service.inc_success_income_requests().await;
                    return Ok(Json(val));
                }
                _ => {}
            }
        }
    }

    monitoring_service.inc_error_income_requests().await;
    Err(ResponseError {
        status: Status::InternalServerError,
        error: format!("failed to request all RPCs for chainId: {chain_id}"),
        internal_error: Err(anyhow!("failed to get rpcs for chainId {chain_id}")),
    })
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct InnerMetricResponse {
    rpc: String,
    metrics: RpcMetrics,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct MetricsResponse {
    rpcs: Vec<InnerMetricResponse>,
}

#[openapi(tag = "Metrics")]
#[get("/v1/chain/<chain_id>/metrics")]
pub async fn get_metrics(
    chain_id: &str,
    evm_rpc_service: &State<Arc<EvmRpcService>>,
    config_repo: &State<ConfigRepo>,
) -> ResponseResult<MetricsResponse> {
    if let None = config_repo
        .supported_chain_ids
        .iter()
        .position(|val| val == chain_id)
    {
        return Err(ResponseError {
            status: Status::BadRequest,
            error: format!("chainId {chain_id} is not supported yet"),
            internal_error: Err(anyhow!("chainId {chain_id} is not supported")),
        });
    }

    let Some(rpcs) = evm_rpc_service.get_rpcs_for_chain_id_cache(chain_id).await else {
        return Err(ResponseError {
            status: Status::InternalServerError,
            error: format!("No rpc provided for chainId {chain_id}"),
            internal_error: Err(anyhow!("failed to get rpcs for chainId {chain_id}")),
        });
    };

    Ok(Json(MetricsResponse {
        rpcs: rpcs
            .iter()
            .map(|val| InnerMetricResponse {
                rpc: val.0.clone(),
                metrics: val.1.clone(),
            })
            .collect(),
    }))
}
