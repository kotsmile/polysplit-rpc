use rocket_jwt::jwt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub email: String,
}
