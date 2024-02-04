use std::sync::Arc;

use rocket::http::{CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;

use crate::models::auth::AuthUser;
use crate::services::jwt::JwtService;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<AuthUser, ()> {
        let jwt_service = request
            .guard::<&State<Arc<JwtService>>>()
            .await
            .expect("jwt service");

        let cookies = request
            .guard::<&CookieJar<'_>>()
            .await
            .expect("request cookies");

        match jwt_service.extract_cookies(cookies) {
            Ok(val) => Outcome::Success(val),
            Err(err) => {
                log::error!("failed to extract cookies: {err}");
                Outcome::Forward(Status::Unauthorized)
            }
        }
    }
}
