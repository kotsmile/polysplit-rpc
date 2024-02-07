use std::sync::Arc;

use anyhow::{anyhow, Context};
use rocket::{get, http::Status, post, State};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::{Group, Rpc},
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
    let group = group_service
        .get_group_by_id(&group_id)
        .await
        .context("failed to find group by id")
        .map_err(|err| ResponseError {
            status: Status::InternalServerError,
            error: format!("Failed to find group"),
            internal_error: Err(err),
        })?;

    let Some(group) = group else {
        return Err(ResponseError {
            status: Status::NotFound,
            error: format!("Failed to find group"),
            internal_error: Err(anyhow!("no group was found for: {group_id}")),
        });
    };

    if group.owner_id != user.id {
        return Err(ResponseError {
            status: Status::Forbidden,
            error: format!("Not a owner of group"),
            internal_error: Err(anyhow!("user {} is not owner of group {group_id}", user.id)),
        });
    }

    group_service
        .get_group_rpcs(&group_id)
        .await
        .context("failed to request rpcs for group")
        .map_err(|err| ResponseError {
            status: Status::InternalServerError,
            error: format!("Failed to find rpcs"),
            internal_error: Err(err),
        })
        .map(ResponseData::build)
}

#[openapi(tag = "Groups")]
#[get("/v2/groups")]
pub async fn get_groups(
    user: UserClaim,
    group_service: &State<Arc<GroupService>>,
) -> ResponseResultData<Vec<Group>> {
    group_service
        .get_groups(&user.id)
        .await
        .context("failed to get all groups for user")
        .map_err(|err| ResponseError {
            error: format!("Failed to retrieve groups"),
            status: Status::InternalServerError,
            internal_error: Err(err),
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
        .context("failed to create group for user")
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
