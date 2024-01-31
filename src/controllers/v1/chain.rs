use std::sync::Arc;

use rocket::{http::Status, post, serde::json::Json, State};

use serde_json::Value;

use crate::{
    repo::config::ConfigRepo,
    services::evm_rpc::EvmRpcService,
    util::controllers::{ResponseError, ResponseResult},
};

#[post("/v1/chain/<chain_id>", format = "json", data = "<rpc_call>")]
pub async fn post_chain_v1(
    chain_id: &str,
    rpc_call: Json<Value>,
    evm_rpc_service: &State<Arc<EvmRpcService>>,
    config_repo: &State<ConfigRepo>,
) -> ResponseResult<Value> {
    if let None = config_repo
        .supported_chain_ids
        .iter()
        .position(|val| val == chain_id)
    {
        log::error!("chainId {chain_id} is not supported");
        return Err(ResponseError {
            status: Status::BadRequest,
            error: format!("chainId {chain_id} is not supported yet"),
        });
    }

    let Some(rpcs) = evm_rpc_service.get_rpcs_for_chain_id(chain_id).await else {
        log::error!("failed to get rpcs for chainId {chain_id}");
        return Err(ResponseError {
            status: Status::InternalServerError,
            error: "Internal error".to_string(),
        });
    };

    let rpc_call = rpc_call.into_inner();
    for rpc in &rpcs {
        let response = evm_rpc_service
            // TODO(@kotsmile): add proxy handling
            .rpc_request(rpc, None, &rpc_call, config_repo.feed_max_timeout)
            .await;

        match response {
            Ok(val) => return Ok(Json(val)),
            // TODO(@kotsmile): add error case
            _ => {}
        }
    }

    Err(ResponseError {
        status: Status::InternalServerError,
        error: format!("failed to request all RPCs for chainId: {chain_id}"),
    })
}
