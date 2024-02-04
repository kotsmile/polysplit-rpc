use anyhow::{Context, Error};
use reqwest::header::AUTHORIZATION;
use rocket::{
    get,
    http::{Cookie, CookieJar, SameSite},
    response::{Debug, Redirect},
};
use rocket_oauth2::{OAuth2, TokenResponse};

use crate::util::oauth2::GoogleUserInfo;

#[get("/auth/google")]
pub async fn get_auth_google(
    token: TokenResponse<GoogleUserInfo>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Debug<Error>> {
    let user_info: GoogleUserInfo = reqwest::Client::builder()
        .build()
        .context("failed to build reqwest client")?
        .get("https://people.googleapis.com/v1/people/me?personFields=names")
        .header(AUTHORIZATION, format!("Bearer {}", token.access_token()))
        .send()
        .await
        .context("failed to complete request")?
        .json()
        .await
        .context("failed to deserialize response")?;

    let real_name = user_info
        .names
        .first()
        .and_then(|n| n.get("displayName"))
        .and_then(|s| s.as_str())
        .unwrap_or("");

    cookies.add_private(
        Cookie::build(("username", real_name.to_string()))
            .same_site(SameSite::Lax)
            .build(),
    );
    Ok(Redirect::to("http://localhost:3000"))
}

#[get("/login/google")]
pub fn get_login_google(oauth2: OAuth2<GoogleUserInfo>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2.get_redirect(cookies, &["profile"]).unwrap()
}
