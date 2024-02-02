#![warn(missing_debug_implementations, rust_2018_idioms)]
use std::sync::Arc;

use anyhow::{anyhow, Result};
use crons::rpc_feed_cron;
use rocket::tokio::{sync::RwLock, task};

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
use services::{
    evm_rpc::{EvmRpcService, Metric},
    monitoring::MonitoringService,
    proxy::ProxyService,
};
use setup::setup_app;

async fn run_tasks(
    evm_rpc_service: Arc<EvmRpcService>,
    proxy_service: Arc<RwLock<ProxyService>>,
    config_repo: ConfigRepo,
) {
    {
        let proxy_service = proxy_service.clone();
        task::spawn(async move {
            let _ = proxy_service
                .write()
                .await
                .init_proxies()
                .await
                .map_err(|err| log::error!("failed to init proxy: {err}"));
        });
    }

    {
        let evm_rpc_service = evm_rpc_service.clone();
        let proxy_service = proxy_service.clone();
        let supported_chain_ids = config_repo.supported_chain_ids.clone();
        let feed_max_timeout = config_repo.feed_max_timeout.clone();
        task::spawn(async move {
            rpc_feed_cron(
                evm_rpc_service,
                proxy_service,
                supported_chain_ids,
                feed_max_timeout,
            )
            .await;
        });
    }

    let chain_to_rpc = evm_rpc_service
        .fetch_rpcs()
        .await
        .map_err(|err| log::error!("failed to fetch rpcs from chainlist: {err}"));
    let Ok(chain_to_rpc) = chain_to_rpc else {
        return;
    };

    for chain_id in &config_repo.supported_chain_ids {
        let Some(rpcs) = chain_to_rpc.get(chain_id) else {
            continue;
        };

        let mut rpcs_for_chain_id: Vec<(String, Metric)> = Vec::new();
        for rpc in rpcs {
            rpcs_for_chain_id.push((
                rpc.clone(),
                Metric {
                    response_time_ms: 0,
                },
            ));
        }

        evm_rpc_service
            .set_rpcs_for_chain_id(chain_id, rpcs_for_chain_id)
            .await;
    }
}

#[rocket::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
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
    ));
    let monitoring_service = Arc::new(MonitoringService::new(cache_repo.clone()));

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
        evm_rpc_service.clone(),
        proxy_service.clone(),
        monitoring_service.clone(),
        config_repo.clone(),
    )
    .launch()
    .await
    .map(|_| {})
    .map_err(|err| anyhow!("failed to start application: {err}"))
}
