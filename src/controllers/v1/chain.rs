use rocket::{post, serde::json::Json};

use serde_json::Value;

#[post("/v1/chain/<chain_id>", format = "json", data = "<rpc_call>")]
pub async fn post_chain_v1(chain_id: &str, rpc_call: Json<Value>) -> Json<Value> {
    println!("{chain_id}");
    return rpc_call;
}
