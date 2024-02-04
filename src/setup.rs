use std::sync::Arc;

use rocket::{catchers, http::Method, routes, tokio::sync::RwLock, Build, Rocket};
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_governor::rocket_governor_catcher;
use rocket_oauth2::OAuth2;
use rocket_okapi::{openapi_get_routes, rapidoc::*, settings::UrlObject, swagger_ui::*};

use crate::controllers::oauth2;
use crate::controllers::status;
use crate::controllers::v1::chain;
use crate::controllers::v1::monitoring;
use crate::models::auth::GoogleUserInfo;
use crate::repo::config::ConfigRepo;
use crate::services::evm_rpc::EvmRpcService;
use crate::services::jwt::JwtService;
use crate::services::monitoring::MonitoringService;
use crate::services::proxy::ProxyService;

pub fn setup_app(
    config_repo: ConfigRepo,
    evm_rpc_service: Arc<EvmRpcService>,
    proxy_service: Arc<RwLock<ProxyService>>,
    monitoring_service: Arc<MonitoringService>,
    jwt_service: Arc<JwtService>,
) -> Rocket<Build> {
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    std::env::set_var("ROCKET_PORT", config_repo.port.to_string());
    std::env::set_var("ROCKET_OAUTH", config_repo.rocket_oauth.to_string());
    println!("{}", std::env::var("ROCKET_OAUTH").unwrap());

    rocket::build()
        .manage(config_repo)
        .manage(evm_rpc_service)
        .manage(proxy_service)
        .manage(monitoring_service)
        .manage(jwt_service)
        // .manage(storage)
        .register("/", catchers!(rocket_governor_catcher))
        .mount(
            "/",
            openapi_get_routes![
                status::get_health,
                chain::get_metrics_v1,
                monitoring::get_monitoring_v1
            ],
        )
        .mount(
            "/",
            routes![
                chain::post_chain_v1,
                oauth2::get_auth_google,
                oauth2::get_login_google
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
        .attach(cors.to_cors().unwrap())
        .attach(OAuth2::<GoogleUserInfo>::fairing("google"))
}
