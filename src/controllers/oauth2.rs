use std::sync::Arc;

use anyhow::{Context, Error};
use rocket::{
    get,
    http::CookieJar,
    response::{Debug, Redirect},
    State,
};
use rocket_oauth2::{OAuth2, TokenResponse};

use crate::{models::auth::GoogleUserInfo, repo::config::ConfigRepo, services::jwt::JwtService};

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

    jwt_service
        .setup_cookies(cookies, user_info.email.to_string())
        .context("failed to setup cookies")?;

    Ok(Redirect::to(config_repo.frontend_url.clone()))
}
