#![warn(missing_debug_implementations, rust_2018_idioms)]
use std::sync::Arc;

use anyhow::{Context, Result};
use rocket::tokio::sync::RwLock;

mod client;
mod controllers;
mod crons;
mod middleware;

mod models;
mod repo;
mod services;

mod setup;
mod tasks;

mod util;

use client::{
    chainlist::ChainlistClient,
    proxyseller::{ProxysellerClient, ProxysellerOrder},
};
use repo::{cache::CacheRepo, config::ConfigRepo, storage::StorageRepo};
use services::{
    evm_rpc::EvmRpcService, jwt::JwtService, monitoring::MonitoringService, proxy::ProxyService,
    user::UserService,
};
use setup::setup_app;
use tasks::run_tasks;

#[rocket::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    env_logger::init();

    let cache_repo = Arc::new(RwLock::new(CacheRepo::new()));
    let config_repo = ConfigRepo::new().context("failed to initiate config repo")?;
    let storage_repo = Arc::new(
        StorageRepo::new(config_repo.database_url.clone(), 5)
            .await
            .context("failed to initiate storage repo")?,
    );

    let proxyseller_client = Box::new(ProxysellerClient::new(
        config_repo.proxyseller_api_key.clone(),
        // TODO(@kotsmile): move orders to envs
        vec![ProxysellerOrder(
            String::from("mix"),
            String::from("1973991"),
        )],
        3000,
    ));
    let chainlist_client = Box::new(ChainlistClient::new());

    let proxy_service = Arc::new(RwLock::new(ProxyService::new(proxyseller_client)));
    let evm_rpc_service = Arc::new(EvmRpcService::new(
        cache_repo.clone(),
        chainlist_client.clone(),
    ));
    let monitoring_service = Arc::new(MonitoringService::new(cache_repo.clone()));
    let jwt_service = Arc::new(JwtService::new());
    let user_service = Arc::new(UserService::new(storage_repo.clone()));

    run_tasks(
        evm_rpc_service.clone(),
        proxy_service.clone(),
        config_repo.clone(),
    )
    .await;

    crons::run_crons(
        evm_rpc_service.clone(),
        proxy_service.clone(),
        config_repo.clone(),
    )
    .await?;

    setup_app(
        config_repo.clone(),
        evm_rpc_service.clone(),
        proxy_service.clone(),
        monitoring_service.clone(),
        jwt_service.clone(),
        user_service.clone(),
    )
    .launch()
    .await
    .map(|_| {})
    .context("failed to start application")
}
