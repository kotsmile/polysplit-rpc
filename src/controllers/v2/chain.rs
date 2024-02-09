use std::sync::Arc;

use anyhow::{anyhow, Context};
use rocket::{get, http::Status, post, serde::json::Json, tokio::sync::RwLock, State};
use rocket_okapi::openapi;
use serde_json::Value;

use crate::{
    client::chainlist::ChainConfig,
    models::Rpc,
    repo::config::ConfigRepo,
    services::{evm_rpc::EvmRpcService, proxy::ProxyService},
    util::controllers::{ResponseData, ResponseError, ResponseResult, ResponseResultData},
};

#[openapi(tag = "Chains")]
#[get("/v2/chains")]
pub async fn get_chains(
    evm_rpc_service: &State<Arc<EvmRpcService>>,
) -> ResponseResultData<Vec<ChainConfig>> {
    evm_rpc_service
        .get_chains()
        .await
        .context("failed to get chains from evm_rpc service")
        .map_err(|err| ResponseError {
            status: Status::InternalServerError,
            error: format!("Failed to fetch chains"),
            internal_error: Err(err),
        })
        .map(ResponseData::build)
}

#[openapi(tag = "Chains")]
#[get("/v2/chain/<chain_id>/rpc")]
pub async fn get_chain_rpc(
    chain_id: &str,
    evm_rpc_service: &State<Arc<EvmRpcService>>,
) -> ResponseResultData<Vec<String>> {
    evm_rpc_service
        .get_public_rpcs_for_chain_id(chain_id)
        .await
        .context("failed to get public rpcs from evm_rpc service")
        .map_err(|err| ResponseError {
            status: Status::InternalServerError,
            error: format!("Internal error"),
            internal_error: Err(err),
        })
        .map(|v| v.iter().map(|el| el.url.clone()).collect())
        .map(ResponseData::build)
}

#[openapi(tag = "Chains")]
#[post("/v2/chain/<chain_id>/<api_key>", format = "json", data = "<rpc_call>")]
pub async fn post_chain(
    chain_id: &str,
    api_key: &str,
    rpc_call: Json<Value>,
    evm_rpc_service: &State<Arc<EvmRpcService>>,
    proxy_service: &State<Arc<RwLock<ProxyService>>>,
    config_repo: &State<ConfigRepo>,
) -> ResponseResult<Value> {
    // TODO add monitoring
    // monitoring_service.inc_income_requests().await;

    if let None = config_repo
        .supported_chain_ids
        .iter()
        .position(|val| val == chain_id)
    {
        // monitoring_service.inc_error_income_requests().await;
        return Err(ResponseError {
            status: Status::BadRequest,
            error: format!("chainId {chain_id} is not supported yet"),
            internal_error: Err(anyhow!("chainId {chain_id} is not supported")),
        });
    }

    let Some(rpcs) = evm_rpc_service
        .get_rpcs_for_api_key_cache(api_key, chain_id)
        .await
    else {
        // monitoring_service.inc_error_income_requests().await;
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
                .rpc_request(
                    &rpc,
                    proxy_config,
                    &rpc_call,
                    config_repo.feed_max_timeout * i,
                )
                .await;

            match response {
                Ok(val) => {
                    log::info!("picked rpc: {}", rpc);
                    // monitoring_service.inc_success_income_requests().await;
                    return Ok(Json(val));
                }
                _ => {}
            }
        }
    }

    // monitoring_service.inc_error_income_requests().await;
    Err(ResponseError {
        status: Status::InternalServerError,
        error: format!("failed to request all RPCs for chainId: {chain_id}"),
        internal_error: Err(anyhow!("failed to get rpcs for chainId {chain_id}")),
    })
}
