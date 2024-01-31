#![warn(missing_debug_implementations, rust_2018_idioms)]
use std::sync::Arc;

use anyhow::{anyhow, Result};
use rocket::tokio::sync::RwLock;

mod client;
mod controllers;
mod models;
mod repo;
mod services;
mod setup;
mod traits;
mod util;

use client::proxyseller::{ProxysellerClient, ProxysellerOrder};
use repo::{cache::CacheRepo, config::ConfigRepo};
use services::proxy::ProxyService;

// use setup::setup_app;

#[rocket::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    env_logger::init();

    let cache_repo = Arc::new(RwLock::new(CacheRepo::new()));
    let config_repo =
        ConfigRepo::new().map_err(|err| anyhow!("failed to inititate config repo: {err}"))?;

    let proxyseller_client = Box::new(ProxysellerClient::new(
        config_repo.proxyseller_api_key,
        // TODO(@kotsmile): move orders to envs
        vec![ProxysellerOrder("mix".to_string(), "1953510".to_string())],
        3000,
    ));

    let proxy_service = Arc::new(RwLock::new(ProxyService::new(proxyseller_client)));
    proxy_service
        .write()
        .await
        .init_proxies()
        .await
        .expect("failed to init proxy");

    proxy_service
        .write()
        .await
        .rotate_proxy()
        .await
        .expect("proxy rotation failed");

    match proxy_service.read().await.get_proxy() {
        Some(config) => println!("{config:?}"),
        None => eprintln!("ERROR: cant find proxy"),
    }

    Ok(())

    // setup_app(/*Box::new(storage) as Box<dyn Storage>*/)
    //     .launch()
    //     .await
    //     .map(|_| {})
    //     .map_err(|err| anyhow!("Cant run application: {err}"))
}
