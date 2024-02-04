use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct GoogleUserInfo {
    pub email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub email: String,
}
