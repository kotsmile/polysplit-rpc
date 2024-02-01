use std::sync::Arc;

use rocket::{routes, tokio::sync::RwLock, Build, Rocket};
use rocket_okapi::{openapi_get_routes, rapidoc::*, settings::UrlObject, swagger_ui::*};

use crate::controllers::status;
use crate::controllers::v1::chain;
use crate::repo::config::ConfigRepo;
use crate::services::evm_rpc::EvmRpcService;
use crate::services::proxy::ProxyService;

pub fn setup_app(
    evm_rpc_service: Arc<EvmRpcService>,
    proxy_service: Arc<RwLock<ProxyService>>,
    config_repo: ConfigRepo,
) -> Rocket<Build> {
    rocket::build()
        .manage(evm_rpc_service)
        .manage(proxy_service)
        .manage(config_repo)
        // .manage(storage)
        .mount("/", openapi_get_routes![status::get_health])
        .mount("/", routes![chain::post_chain_v1])
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
}
