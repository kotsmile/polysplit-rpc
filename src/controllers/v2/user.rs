use std::sync::Arc;

use anyhow::{anyhow, Context};
use rocket::{get, http::Status, State};
use rocket_okapi::openapi;

use crate::{
    models::User,
    services::{jwt::UserClaim, user::UserService},
    util::controllers::{ResponseData, ResponseError, ResponseResultData},
};

#[openapi(tag = "User")]
#[get("/v2/user/me")]
pub async fn get_user_me(
    user: UserClaim,
    user_service: &State<Arc<UserService>>,
) -> ResponseResultData<User> {
    let user = user_service
        .get_user_by_email(&user.email)
        .await
        .context("failed to find user in group service")
        .map_err(|err| ResponseError {
            error: "Failed to find user".to_string(),
            status: Status::InternalServerError,
            internal_error: Err(err),
        })?
        .ok_or(ResponseError {
            error: "Failed to find user".to_string(),
            status: Status::InternalServerError,
            internal_error: Err(anyhow!("no user was found")),
        })?;

    Ok(ResponseData::build(user))
}
