use std::sync::Arc;

use anyhow::{Context, Error};
use rocket::{
    get,
    http::CookieJar,
    response::{Debug, Redirect},
    State,
};
use rocket_oauth2::{OAuth2, TokenResponse};
use serde::Deserialize;

use crate::{
    models::user::NewUser,
    repo::config::ConfigRepo,
    services::{jwt::JwtService, user::UserService},
};

#[derive(Deserialize, Debug)]
pub struct GoogleUserInfo {
    pub email: String,
}

#[get("/login/google")]
pub fn get_login_google(oauth2: OAuth2<GoogleUserInfo>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2
        .get_redirect(cookies, &["https://www.googleapis.com/auth/userinfo.email"])
        .unwrap()
}

#[get("/auth/google")]
pub async fn get_auth_google(
    token: TokenResponse<GoogleUserInfo>,
    cookies: &CookieJar<'_>,
    jwt_service: &State<Arc<JwtService>>,
    config_repo: &State<ConfigRepo>,
    user_service: &State<Arc<UserService>>,
) -> Result<Redirect, Debug<Error>> {
    let user_info: GoogleUserInfo = reqwest::Client::builder()
        .build()
        .context("failed to build reqwest client")?
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .query(&[("access_token", token.access_token())])
        .send()
        .await
        .context("failed to complete request")?
        .json()
        .await
        .context("failed to deserialize response")?;

    let user = user_service
        .get_user_by_email(&user_info.email)
        .await
        .context("failed to find user")?;

    if let None = user {
        user_service
            .create_user(&NewUser {
                email: user_info.email.clone(),
            })
            .await
            .context("failed to find user")?;

        // TODO: in future this branch will redirect to register page
        // Ok(Redirect::to(config_repo.frontend_url.clone()))
    }

    jwt_service
        .setup_cookies(cookies, user_info.email.to_string())
        .context("failed to setup cookies")?;

    Ok(Redirect::to(config_repo.frontend_url.clone()))
}
