use std::{cmp::Ordering, collections::HashMap, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use rocket::tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    repo::config::ConfigRepo,
    services::{
        evm_rpc::{EvmRpcService, RpcMetrics},
        proxy::ProxyService,
    },
};

const BATCH_SIZE: usize = 100;

pub async fn run_crons(
    evm_rpc_service: Arc<EvmRpcService>,
    proxy_service: Arc<RwLock<ProxyService>>,
    config_repo: ConfigRepo,
) -> Result<()> {
    let sched = JobScheduler::new().await?;

    {
        let proxy_service = proxy_service.clone();
        sched
            .add(Job::new_async("0 */5 * * * *", move |_uuid, mut _l| {
                let evm_rpc_service = evm_rpc_service.clone();
                let proxy_service = proxy_service.clone();
                let supported_chain_ids = config_repo.supported_chain_ids.clone();
                let feed_max_timeout = config_repo.feed_max_timeout.clone();

                Box::pin(async move {
                    log::info!("start rpc feed cron");
                    rpc_feed_cron(
                        evm_rpc_service,
                        proxy_service,
                        supported_chain_ids,
                        feed_max_timeout,
                    )
                    .await;
                })
            })?)
            .await?;
    }

    {
        sched
            .add(Job::new_async("0 */15 * * * *", move |_uuid, mut _l| {
                let proxy_service = proxy_service.clone();
                Box::pin(async move {
                    log::info!("start proxy updater cron");
                    proxy_updater_cron(proxy_service).await;
                })
            })?)
            .await?;
    }

    sched.start().await?;

    Ok(())
}

pub async fn proxy_updater_cron(proxy_service: Arc<RwLock<ProxyService>>) {
    let _ = proxy_service
        .write()
        .await
        .rotate_proxy()
        .await
        .map_err(|err| log::error!("failed to rotate proxy: {err}"));
}

pub async fn rpc_feed_cron(
    evm_rpc_service: Arc<EvmRpcService>,
    proxy_service: Arc<RwLock<ProxyService>>,
    supported_chain_ids: Vec<String>,
    feed_max_timeout: Duration,
) {
    for chain_id in &supported_chain_ids {
        let rpcs = evm_rpc_service
            .get_rpcs_by_chain_id(chain_id)
            .await
            .context("failed to get rpcs for chain_id");
        let Ok(rpcs) = rpcs else {
            log::error!("failed to find rpcs for chain id");
            continue;
        };

        log::debug!("rpc length for {chain_id}: {}", rpcs.len());

        let mut rpc_to_metric: HashMap<String, Result<RpcMetrics>> = HashMap::new();
        for batch in rpcs.chunks(BATCH_SIZE) {
            let proxy_service = proxy_service.read().await;
            let proxy_config = proxy_service.get_proxy();
            let mut futures = FuturesUnordered::new();
            for rpc in batch {
                let evm_rpc_service_clone = evm_rpc_service.clone();
                let rpc_clone = rpc.to_owned();

                futures.push(async move {
                    let metric = evm_rpc_service_clone
                        .rpc_health_check(
                            chain_id,
                            &rpc.url,
                            proxy_config,
                            feed_max_timeout,
                            // TODO(@kotsmile): remove hard code
                            3,
                        )
                        .await;
                    (rpc_clone, metric)
                });
            }

            while let Some((rpc, metric)) = futures.next().await {
                rpc_to_metric.insert(rpc.url, metric);
            }
        }

        let mut rpcs = Vec::from_iter(rpc_to_metric.iter());
        rpcs.sort_by(|&(_, a), &(_, b)| {
            let Ok(a) = a else {
                return Ordering::Greater;
            };
            let Ok(b) = b else {
                return Ordering::Less;
            };
            b.to_score().total_cmp(&a.to_score())
        });

        let rpcs: Vec<(String, RpcMetrics)> = rpcs
            .iter()
            .filter_map(|&(rpc, metric)| {
                let Ok(metric) = metric else {
                    return None;
                };

                Some((rpc.clone(), metric.clone()))
            })
            .collect();

        evm_rpc_service.set_rpcs_for_chain_id(chain_id, rpcs).await;
    }
}
