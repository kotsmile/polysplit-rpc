#![warn(missing_debug_implementations, rust_2018_idioms)]
use std::{sync::Arc, thread::spawn};

use anyhow::{anyhow, Result};
use libs::rpc_feed::RpcFeedLib;
use rocket::tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};

mod client;
mod controllers;
mod libs;
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

// use setup::setup_app;

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
        vec![ProxysellerOrder("mix".to_string(), "1953510".to_string())],
        3000,
    ));
    let chainlist_client = Box::new(ChainlistClient::new());

    let proxy_service = Arc::new(RwLock::new(ProxyService::new(proxyseller_client)));
    let evm_rpc_service = Arc::new(EvmRpcService::new(
        cache_repo.clone(),
        chainlist_client.clone(),
        config_repo.supported_chain_ids.clone(),
    ));
    let rpc_feed_lib = Arc::new(RpcFeedLib::new(
        evm_rpc_service.clone(),
        proxy_service.clone(),
        config_repo.supported_chain_ids.clone(),
        config_repo.feed_max_timeout.clone(),
    ));

    let sched = JobScheduler::new().await?;

    {
        let rpc_feed_lib = rpc_feed_lib.clone();
        sched
            .add(Job::new_async("0 */1 * * * *", move |_uuid, mut _l| {
                let rpc_feed_lib = rpc_feed_lib.clone();
                Box::pin(async move {
                    log::info!("start rpc feed cron");
                    rpc_feed_lib.rpc_feed_cron().await;
                })
            })?)
            .await?;
    }
    // TODO(@kotsmile): add task for proxy fetcher;

    sched.start().await?;

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
