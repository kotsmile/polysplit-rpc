use std::time::SystemTime;

use anyhow::{bail, Context, Result};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use rocket::{
    http::{Cookie, CookieJar, SameSite},
    serde,
};
use serde::{Deserialize, Serialize};

use crate::models::auth::AuthUser;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MetaUserJwt {
    pub user: AuthUser,
    pub exp: u64,
}

pub struct JwtService {
    secret_key: String,
    access_expiration: u64,
}

const ACCESS_TOKEN_KEY: &'static str = "access_token";

impl JwtService {
    pub fn new(secret_key: String, access_expiration: u64) -> Self {
        Self {
            secret_key,
            access_expiration,
        }
    }

    pub fn encode_jwt_user(&self, user_jwt: AuthUser) -> Result<String> {
        let meta_user_jwt = MetaUserJwt {
            user: user_jwt,
            exp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + self.access_expiration,
        };

        encode(
            &Header::default(),
            &meta_user_jwt,
            &EncodingKey::from_secret(self.secret_key.as_ref()),
        )
        .context("faield to sign user structure")
    }

    pub fn decode_jwt_user(&self, jwt: String) -> Result<AuthUser> {
        let user_jwt: TokenData<MetaUserJwt> = decode(
            &jwt,
            &DecodingKey::from_secret(self.secret_key.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .context("failed to decode user structure")?;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if now > user_jwt.claims.exp {
            bail!("Jwt is expired");
        }

        Ok(user_jwt.claims.user)
    }

    pub fn setup_cookies(&self, cookies: &CookieJar<'_>, email_address: String) -> Result<()> {
        let access_token = self
            .encode_jwt_user(AuthUser {
                email: email_address.to_string(),
            })
            .context("failed to encode jwt")?;
        println!("{access_token}");

        cookies.add_private(
            Cookie::build((ACCESS_TOKEN_KEY, access_token))
                .same_site(SameSite::Lax)
                .build(),
        );

        Ok(())
    }

    pub fn extract_cookies(&self, cookies: &CookieJar<'_>) -> Result<AuthUser> {
        cookies
            .get_private(ACCESS_TOKEN_KEY)
            .context("failed to get access token")
            .and_then(|val| self.decode_jwt_user(val.to_string()))
    }
}
