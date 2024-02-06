use anyhow::Result;
use lazy_static::lazy_static;
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket_jwt::jwt;
use rocket_okapi::request::OpenApiFromRequest;
use uuid::Uuid;

pub struct JwtService;

const ACCESS_TOKEN_KEY: &'static str = "access_token";

lazy_static! {
    static ref JWT_SECRET_KEY: String = {
        let _ = dotenvy::dotenv();
        std::env::var("JWT_SECRET_KEY").unwrap()
    };
    static ref JWT_EXPIRATION_ACCESS: u32 = {
        let _ = dotenvy::dotenv();
        std::env::var("JWT_EXPIRATION_ACCESS")
            .unwrap()
            .parse::<u32>()
            .unwrap()
    };
}

#[jwt(JWT_SECRET_KEY, exp = 10000, cookie = "access_token")]
pub struct UserClaim {
    pub id: Uuid,
    pub email: String,
}

impl JwtService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn setup_cookies(&self, cookies: &CookieJar<'_>, email: String, id: &Uuid) -> Result<()> {
        let access_token = UserClaim::sign(UserClaim {
            email,
            id: id.clone(),
        });

        cookies.add(
            Cookie::build((ACCESS_TOKEN_KEY, format!("Bearer {access_token}")))
                .same_site(SameSite::Lax)
                .build(),
        );

        Ok(())
    }
}

impl OpenApiFromRequest<'_> for UserClaim {
    fn from_request_input(
        _gen: &mut rocket_okapi::gen::OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<rocket_okapi::request::RequestHeaderInput> {
        rocket_okapi::Result::Ok(rocket_okapi::request::RequestHeaderInput::None)
    }

    fn get_responses(
        _gen: &mut rocket_okapi::gen::OpenApiGenerator,
    ) -> rocket_okapi::Result<rocket_okapi::okapi::openapi3::Responses> {
        Ok(rocket_okapi::okapi::openapi3::Responses::default())
    }
}

// impl<'r> OpenApiFromRequest<'r> for MyDB {
//     fn from_request_input(
//         _gen: &mut OpenApiGenerator,
//         _name: String,
//         _required: bool,
//     ) -> rocket_okapi::Result<RequestHeaderInput> {
//         Ok(RequestHeaderInput::None)
//     }
// }
