#![warn(missing_debug_implementations, rust_2018_idioms)]
use std::sync::Arc;

use anyhow::{anyhow, Result};
use rocket::tokio::sync::RwLock;

mod client;
mod controllers;
mod crons;
mod models;
mod repo;
mod services;
mod setup;
mod util;

use client::{
    chainlist::ChainlistClient,
    proxyseller::{ProxysellerClient, ProxysellerOrder},
};
use repo::{cache::CacheRepo, config::ConfigRepo};
use services::{evm_rpc::EvmRpcService, proxy::ProxyService};
use setup::setup_app;

#[rocket::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    env_logger::init();

    let cache_repo = Arc::new(RwLock::new(CacheRepo::new()));
    let config_repo =
        ConfigRepo::new().map_err(|err| anyhow!("failed to inititate config repo: {err}"))?;

    let proxyseller_client = Box::new(ProxysellerClient::new(
        config_repo.proxyseller_api_key.clone(),
        // TODO(@kotsmile): move orders to envs
        vec![ProxysellerOrder(
            String::from("mix"),
            String::from("1953510"),
        )],
        3000,
    ));
    let chainlist_client = Box::new(ChainlistClient::new());

    let proxy_service = Arc::new(RwLock::new(ProxyService::new(proxyseller_client)));
    let evm_rpc_service = Arc::new(EvmRpcService::new(
        cache_repo.clone(),
        chainlist_client.clone(),
        config_repo.supported_chain_ids.clone(),
    ));

    crons::run_crons(
        evm_rpc_service.clone(),
        proxy_service.clone(),
        config_repo.clone(),
    )
    .await?;

    setup_app(
        evm_rpc_service.clone(),
        proxy_service.clone(),
        config_repo.clone(),
    )
    .launch()
    .await
    .map(|_| {})
    .map_err(|err| anyhow!("failed to start application: {err}"))
}
