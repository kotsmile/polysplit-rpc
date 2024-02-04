use rocket::get;
use rocket_okapi::openapi;

use crate::{
    services::jwt::UserClaim,
    util::controllers::{ResponseData, ResponseResultData},
};

#[openapi(tag = "User")]
#[get("/v2/user/me")]
pub fn get_user_me(user: UserClaim) -> ResponseResultData<String> {
    Ok(ResponseData::build(user.email))
}
