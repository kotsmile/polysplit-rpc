pub mod monitoring;
pub mod proxy;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Chain {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct NewUser {
    pub email: String,
}

// id uuid not null unique,
//
// name varchar not null,
// owner_id uuid not null,
// api_key text not null,
//
// primary key (id),
// constraint fk_owner foreign key(owner_id) references users(id)

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub api_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct NewGroup {
    pub name: String,
    pub owner_id: Uuid,
}

#[derive(Debug, JsonSchema, Serialize, Deserialize, Clone, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "rpc_visibility", rename_all = "lowercase")]
pub enum RpcVisibility {
    Public,
    Private,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Rpc {
    pub id: i32,
    pub chain_id: String,
    pub url: String,
    pub visibility: RpcVisibility,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct NewRpc {
    pub chain_id: String,
    pub url: String,
    pub visibility: RpcVisibility,
}
