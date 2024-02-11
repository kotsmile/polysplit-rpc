use std::sync::Arc;

use anyhow::{anyhow, Context};
use rocket::{
    get,
    http::{CookieJar, Status},
    response::Redirect,
    State,
};
use rocket_oauth2::{OAuth2, TokenResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    models::NewUser,
    repo::config::ConfigRepo,
    services::{
        jwt::{JwtService, UserClaim},
        user::UserService,
    },
    util::controllers::{ResponseData, ResponseError, ResponseResultData},
};

#[derive(Deserialize, Debug)]
pub struct GoogleUserInfo {
    pub email: String,
}

#[get("/auth/refresh")]
pub fn get_refresh_token(
    cookies: &CookieJar<'_>,
    jwt_service: &State<Arc<JwtService>>,
    config_repo: &State<ConfigRepo>,
) -> Result<Redirect, ResponseError> {
    jwt_service
        .setup_cookies_with_refresh(cookies)
        .context("failed to setup cookies with refresh in jwt service")
        .map_err(|err| ResponseError {
            error: String::from("Failed to setup cookies"),
            status: Status::InternalServerError,
            internal_error: Err(err),
        })?;

    Ok(Redirect::to(config_repo.frontend_url_profile.clone()))
}

#[get("/auth/login/google")]
pub fn get_login_google(oauth2: OAuth2<GoogleUserInfo>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2
        .get_redirect(cookies, &["https://www.googleapis.com/auth/userinfo.email"])
        .unwrap()
}

#[get("/auth/provider/google")]
pub async fn get_provider_google(
    token: TokenResponse<GoogleUserInfo>,
    cookies: &CookieJar<'_>,
    jwt_service: &State<Arc<JwtService>>,
    config_repo: &State<ConfigRepo>,
    user_service: &State<Arc<UserService>>,
) -> Result<Redirect, ResponseError> {
    let user_info: GoogleUserInfo = reqwest::Client::builder()
        .build()
        .context("failed to build reqwest client for google api")
        .map_err(|err| ResponseError {
            error: String::from("Failed to request google api"),
            status: Status::InternalServerError,
            internal_error: Err(err),
        })?
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .query(&[("access_token", token.access_token())])
        .send()
        .await
        .context("failed to complete request to google api")
        .map_err(|err| ResponseError {
            error: String::from("Failed to request google api"),
            status: Status::InternalServerError,
            internal_error: Err(err),
        })?
        .json()
        .await
        .context("failed to deserialize response from google api")
        .map_err(|err| ResponseError {
            error: String::from("Failed to request google api"),
            status: Status::InternalServerError,
            internal_error: Err(err),
        })?;

    let user = user_service
        .get_user_by_email(&user_info.email)
        .await
        .context("failed to find user in user service")
        .map_err(|err| ResponseError {
            error: String::from("Internal error"),
            status: Status::InternalServerError,
            internal_error: Err(err),
        })?;

    let user_id: Uuid = match user {
        Some(user) => Ok::<Uuid, ResponseError>(user.id),
        None => {
            let user = user_service
                .create_user(&NewUser {
                    email: user_info.email.clone(),
                })
                .await
                .context("failed to find user in user service")
                .map_err(|err| ResponseError {
                    error: String::from("Internal error"),
                    status: Status::InternalServerError,
                    internal_error: Err(err),
                })?;

            let Some(user) = user else {
                return Err(ResponseError {
                    error: String::from("Internal error"),
                    status: Status::InternalServerError,
                    internal_error: Err(anyhow!("failed to find user after inserting it")),
                });
            };

            // TODO: in future this branch will redirect to register page
            // Ok(Redirect::to(config_repo.frontend_url.clone()))
            //
            Ok(user.id)
        }
    }?;

    jwt_service
        .setup_cookies(cookies, user_info.email.to_string(), &user_id)
        .context("failed to setup cookies in jwt service")
        .map_err(|err| ResponseError {
            error: String::from("Failed to setup cookies"),
            status: Status::InternalServerError,
            internal_error: Err(err),
        })?;

    Ok(Redirect::to(config_repo.frontend_url_profile.clone()))
}

#[get("/auth/logout")]
pub fn get_logout(
    cookies: &CookieJar<'_>,
    jwt_service: &State<Arc<JwtService>>,
    _user: UserClaim,
) -> ResponseResultData<String> {
    jwt_service
        .clear_cookies(cookies)
        .context("failed to clear cookies")
        .map_err(|err| ResponseError {
            status: Status::InternalServerError,
            error: format!("Internal error"),
            internal_error: Err(err),
        })?;

    Ok(ResponseData::build("success".to_string()))
}
