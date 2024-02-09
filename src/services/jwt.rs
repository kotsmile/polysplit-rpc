use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket_jwt::jwt;
use rocket_okapi::request::OpenApiFromRequest;
use uuid::Uuid;

lazy_static! {
    static ref ACCESS_TOKEN_PKEY: String = {
        let _ = dotenvy::dotenv();
        std::env::var("ACCESS_TOKEN_PKEY").unwrap()
    };
    static ref REFRESH_TOKEN_PKEY: String = {
        let _ = dotenvy::dotenv();
        std::env::var("REFRESH_TOKEN_PKEY").unwrap()
    };
}

#[jwt(ACCESS_TOKEN_PKEY, exp = 86_400 /* 24 hours */, cookie = "access_token")]
pub struct UserClaim {
    pub id: Uuid,
    pub email: String,
}

#[jwt(REFRESH_TOKEN_PKEY, exp = 604_800 /* 7 days */)]
pub struct RefreshClaim {
    pub id: Uuid,
    pub email: String,
}

const ACCESS_TOKEN_COOKIE: &'static str = "access_token";
const REFRESH_TOKEN_COOKIE: &'static str = "refresh_token";

pub struct JwtService;

impl JwtService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn setup_cookies(&self, cookies: &CookieJar<'_>, email: String, id: &Uuid) -> Result<()> {
        let access_token = UserClaim::sign(UserClaim {
            email: email.clone(),
            id: id.clone(),
        });

        let refresh_token = RefreshClaim::sign(RefreshClaim {
            email,
            id: id.clone(),
        });

        cookies.add(
            Cookie::build((ACCESS_TOKEN_COOKIE, format!("Bearer {access_token}")))
                .same_site(SameSite::Lax)
                .build(),
        );

        cookies.add_private(
            Cookie::build((REFRESH_TOKEN_COOKIE, refresh_token))
                .same_site(SameSite::Lax)
                .build(),
        );

        Ok(())
    }

    pub fn setup_cookies_with_refresh(&self, cookies: &CookieJar<'_>) -> Result<()> {
        let refresh_token = cookies
            .get_private(REFRESH_TOKEN_COOKIE)
            .ok_or(anyhow!("refresh token is not exist"))?;

        let refresh_claim = RefreshClaim::decode(refresh_token.value().to_string())
            .context("failed to recover refresh token")?;

        self.setup_cookies(cookies, refresh_claim.user.email, &refresh_claim.user.id)
            .context("failed to setup cookies")
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
