use rocket::get;
use rocket_okapi::openapi;

use crate::util::controllers::{ResponseData, ResponseResultData};

#[openapi(tag = "Status")]
#[get("/status/health")]
pub fn get_health() -> ResponseResultData<String> {
    Ok(ResponseData::build(String::from("healthy")))
}
