use rocket::{get, serde::json::Json};
use rocket_okapi::openapi;

use super::util::ResponseResult;

#[openapi(tag = "Status")]
#[get("/status/health")]
pub fn get_health() -> ResponseResult<()> {
    Ok(Json(()))
}
