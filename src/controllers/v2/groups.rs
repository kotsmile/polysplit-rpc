use std::sync::Arc;

use anyhow::{anyhow, Context};
use rocket::{get, http::Status, patch, post, State};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
    models::{Group, NewRpc, Rpc, RpcVisibility},
    services::{group::GroupService, jwt::UserClaim},
    util::controllers::{RequestResult, ResponseData, ResponseError, ResponseResultData},
};

#[openapi(tag = "Groups")]
#[get("/v2/groups/<group_id>/rpcs")]
pub async fn get_group_rpcs(
    group_id: Uuid,
    user: UserClaim,
    group_service: &State<Arc<GroupService>>,
) -> ResponseResultData<Vec<Rpc>> {
    let _ = group_service
        .get_group_with_owner(&user.id, &group_id)
        .await
        .context("failed to find group with owner id in group service")
        .map_err(|err| ResponseError {
            status: Status::NotFound,
            error: format!("Failed to find group"),
            internal_error: Err(anyhow!("no group was found for: {group_id}: {err}")),
        })?;

    group_service
        .get_group_rpcs(&group_id)
        .await
        .context("failed to request rpcs for group in group service")
        .map_err(|err| ResponseError {
            status: Status::InternalServerError,
            error: format!("Failed to find rpcs"),
            internal_error: Err(err),
        })
        .map(ResponseData::build)
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddGroupRpcRequest {
    pub chain_id: String,
    pub url: String,
}

#[openapi(tag = "Groups")]
#[post("/v2/groups/<group_id>/rpcs", data = "<new_rpc>")]
pub async fn post_group_rpc(
    group_id: Uuid,
    new_rpc: RequestResult<'_, AddGroupRpcRequest>,
    user: UserClaim,
    group_service: &State<Arc<GroupService>>,
) -> ResponseResultData<Rpc> {
    let new_rpc = new_rpc?.into_inner();
    match Url::parse(&new_rpc.url) {
        Ok(url_) => {
            if url_.scheme() != "https" {
                return Err(ResponseError {
                    error: format!("Unsupported schema url"),
                    status: Status::BadRequest,
                    internal_error: Err(anyhow!("unsupported schema")),
                });
            }
        }
        Err(err) => {
            return Err(ResponseError {
                error: format!("Bad format of url: {err}"),
                status: Status::BadRequest,
                internal_error: Err(err).context("failed to parse url"),
            });
        }
    }

    let _ = group_service
        .get_group_with_owner(&user.id, &group_id)
        .await
        .context("no group with owner id in group service")
        .map_err(|err| ResponseError {
            status: Status::NotFound,
            error: format!("Failed to find group"),
            internal_error: Err(anyhow!("no group was found for: {group_id}: {err}")),
        })?;

    group_service
        .add_rpc_to_group(
            &group_id,
            &NewRpc {
                chain_id: new_rpc.chain_id,
                url: new_rpc.url,
                visibility: RpcVisibility::Private,
            },
        )
        .await
        .context("failed to add rpc to group in group service")
        .map_err(|err| ResponseError {
            internal_error: Err(err),
            status: Status::InternalServerError,
            error: format!("Failed to add rpc to group"),
        })
        .map(ResponseData::build)
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateApiKeyResponse {
    pub api_key: String,
}

#[openapi(tag = "Groups")]
#[patch("/v2/groups/<group_id>/api_key")]
pub async fn update_group_api_key(
    group_id: Uuid,
    user: UserClaim,
    group_service: &State<Arc<GroupService>>,
) -> ResponseResultData<UpdateApiKeyResponse> {
    let _ = group_service
        .get_group_with_owner(&user.id, &group_id)
        .await
        .context("failed to find group with owner id in group service")
        .map_err(|err| ResponseError {
            status: Status::NotFound,
            error: format!("Failed to find group"),
            internal_error: Err(anyhow!("no group was found for: {group_id}: {err}")),
        })?;

    let api_key = group_service
        .update_api_key(&group_id)
        .await
        .context("failed to update api key in group service")
        .map_err(|err| ResponseError {
            status: Status::InternalServerError,
            error: format!("Internal error"),
            internal_error: Err(err),
        })?;

    Ok(ResponseData::build(UpdateApiKeyResponse { api_key }))
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResponseGroup {
    pub id: Uuid,
    pub name: String,
}

#[openapi(tag = "Groups")]
#[get("/v2/groups")]
pub async fn get_groups(
    user: UserClaim,
    group_service: &State<Arc<GroupService>>,
) -> ResponseResultData<Vec<ResponseGroup>> {
    group_service
        .get_groups_for_user(&user.id)
        .await
        .context("failed to get all groups for user in group service")
        .map_err(|err| ResponseError {
            error: format!("Failed to retrieve groups"),
            status: Status::InternalServerError,
            internal_error: Err(err),
        })
        .map(|v| {
            v.iter()
                .map(|g| ResponseGroup {
                    id: g.id,
                    name: g.name.clone(),
                })
                .collect()
        })
        .map(ResponseData::build)
}

#[openapi(tag = "Groups")]
#[get("/v2/groups/<group_id>")]
pub async fn get_group_id(
    group_id: Uuid,
    user: UserClaim,
    group_service: &State<Arc<GroupService>>,
) -> ResponseResultData<Group> {
    group_service
        .get_group_with_owner(&user.id, &group_id)
        .await
        .context("failed to find group with owner id in group service")
        .map_err(|err| ResponseError {
            status: Status::NotFound,
            error: format!("Failed to find group"),
            internal_error: Err(anyhow!("no group was found for: {group_id}: {err}")),
        })
        .map(ResponseData::build)
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateGroupResponse {
    pub name: String,
}

#[openapi(tag = "Groups")]
#[post("/v2/groups", data = "<new_group>")]
pub async fn post_group(
    new_group: RequestResult<'_, CreateGroupResponse>,
    user: UserClaim,
    group_service: &State<Arc<GroupService>>,
) -> ResponseResultData<Group> {
    let new_group = new_group?.into_inner();

    group_service
        .create_group(&user.id, &new_group.name)
        .await
        .context("failed to create group for user in group service")
        .map_err(|err| ResponseError {
            error: format!("Failed to create new group"),
            status: Status::InternalServerError,
            internal_error: Err(err),
        })
        .and_then(|val| {
            val.ok_or(ResponseError {
                error: format!("Failed to create new group"),
                status: Status::InternalServerError,
                internal_error: Err(anyhow!("failed to find created group")),
            })
        })
        .map(ResponseData::build)
}
