use rocket::http::{CookieJar, Status};
use rocket::request;
use serde::Deserialize;
use serde_json::Value;

pub struct User {
    pub username: String,
}

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for User {
    type Error = ();

    async fn from_request(request: &'r request::Request<'_>) -> request::Outcome<User, ()> {
        let cookies = request
            .guard::<&CookieJar<'_>>()
            .await
            .expect("request cookies");
        if let Some(cookie) = cookies.get_private("username") {
            return request::Outcome::Success(User {
                username: cookie.value().to_string(),
            });
        }

        request::Outcome::Forward(Status::Unauthorized)
    }
}

// #[derive(Deserialize)]
// pub struct GitHubUserInfo {
//     #[serde(default)]
//     pub name: String,
// }

#[derive(Deserialize)]
pub struct GoogleUserInfo {
    pub names: Vec<Value>,
}

// #[derive(Deserialize)]
// pub struct MicrosoftUserInfo {
//     #[serde(default, rename = "displayName")]
//     pub display_name: String,
// }
