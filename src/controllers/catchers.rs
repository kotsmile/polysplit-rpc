use rocket::{catch, response::Redirect, Request, State};

use crate::repo::config::ConfigRepo;

#[catch(403)]
pub async fn not_authenticated(req: &Request<'_>) -> Redirect {
    let config_repo = req
        .guard::<&State<ConfigRepo>>()
        .await
        .expect("ConfigRepo is managed");

    Redirect::to(config_repo.frontend_url_sign_in.to_string())
}
