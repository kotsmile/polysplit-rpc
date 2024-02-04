use rocket_jwt::jwt;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct GoogleUserInfo {
    pub email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub email: String,
}

#[jwt("secret", exp = 10)]
pub struct UserClaim {
    pub email: String,
}
