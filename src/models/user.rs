use rocket::serde::uuid::Uuid;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct NewUser {
    pub email: String,
}
