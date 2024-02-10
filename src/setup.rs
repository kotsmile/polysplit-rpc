use std::sync::Arc;

use rocket::{catchers, http::Method, routes, Build, Rocket};
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_governor::rocket_governor_catcher;
use rocket_oauth2::OAuth2;
use rocket_okapi::{openapi_get_routes, rapidoc::*, settings::UrlObject, swagger_ui::*};

use crate::controllers::catchers::not_authenticated;
use crate::controllers::oauth2;
use crate::controllers::status;
use crate::controllers::{v1, v2};
use crate::repo::config::ConfigRepo;
use crate::services::group::GroupService;
use crate::services::{
    evm_rpc::EvmRpcService, jwt::JwtService, monitoring::MonitoringService, proxy::ProxyService,
    user::UserService,
};

pub fn setup_app(
    config_repo: ConfigRepo,
    evm_rpc_service: Arc<EvmRpcService>,
    proxy_service: Arc<ProxyService>,
    monitoring_service: Arc<MonitoringService>,
    jwt_service: Arc<JwtService>,
    user_service: Arc<UserService>,
    group_service: Arc<GroupService>,
) -> Rocket<Build> {
    std::env::set_var("ROCKET_PORT", config_repo.port.to_string());
    std::env::set_var("ROCKET_OAUTH", config_repo.rocket_oauth.to_string());
    std::env::set_var("ROCKET_SECRET_KEY", config_repo.secret_key.to_string());
    rocket::build()
        .manage(config_repo)
        .manage(evm_rpc_service)
        .manage(proxy_service)
        .manage(monitoring_service)
        .manage(jwt_service)
        .manage(user_service)
        .manage(group_service)
        // .manage(storage)
        .register("/", catchers![rocket_governor_catcher, not_authenticated])
        .mount(
            "/",
            openapi_get_routes![
                status::get_health,
                // v1
                v1::chain::get_chains,
                v1::chain::get_metrics,
                v1::monitoring::get_monitoring,
                // v2
                // public
                v2::chain::get_chains,
                v2::chain::get_chain_rpc,
                // private
                v2::user::get_user_me,
                v2::groups::get_groups,
                v2::groups::post_group,
                v2::groups::get_group_rpcs,
                v2::groups::post_group_rpc,
                v2::groups::update_group_api_key
            ],
        )
        .mount(
            "/",
            routes![
                oauth2::get_refresh_token,
                oauth2::get_provider_google,
                oauth2::get_login_google,
                // v1
                v1::chain::post_chain,
                // v2
                v2::chain::post_chain
            ],
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(
                &(SwaggerUIConfig {
                    url: "/openapi.json".to_owned(),
                    ..Default::default()
                }),
            ),
        )
        .mount(
            "/rapidoc/",
            make_rapidoc(
                &(RapiDocConfig {
                    general: GeneralConfig {
                        spec_urls: vec![UrlObject::new("General", "/openapi.json")],
                        ..Default::default()
                    },
                    hide_show: HideShowConfig {
                        allow_spec_url_load: false,
                        allow_spec_file_load: false,
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ),
        )
        .attach(
            CorsOptions::default()
                .allowed_origins(AllowedOrigins::all())
                .allowed_methods(
                    vec![Method::Get, Method::Post, Method::Patch]
                        .into_iter()
                        .map(From::from)
                        .collect(),
                )
                .allow_credentials(true)
                .to_cors()
                .unwrap(),
        )
        .attach(OAuth2::<oauth2::GoogleUserInfo>::fairing("google"))
}
