use std::{cmp::Ordering, collections::HashMap, sync::Arc, time::Duration};

use anyhow::Result;
use rocket::tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    repo::config::ConfigRepo,
    services::{
        evm_rpc::{EvmRpcService, Metric, RpcMetric},
        proxy::ProxyService,
    },
};

pub async fn run_crons(
    evm_rpc_service: Arc<EvmRpcService>,
    proxy_service: Arc<RwLock<ProxyService>>,
    config_repo: ConfigRepo,
) -> Result<()> {
    let sched = JobScheduler::new().await?;

    {
        let proxy_service = proxy_service.clone();
        sched
            .add(Job::new_async("0 */1 * * * *", move |_uuid, mut _l| {
                let evm_rpc_service = evm_rpc_service.clone();
                let proxy_service = proxy_service.clone();
                let supported_chain_ids = config_repo.supported_chain_ids.clone();
                let feed_max_timeout = config_repo.feed_max_timeout.clone();

                Box::pin(async move {
                    log::info!("start rpc feed cron");
                    rpc_feed_cron(
                        evm_rpc_service.clone(),
                        proxy_service.clone(),
                        supported_chain_ids.clone(),
                        feed_max_timeout.clone(),
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
                    proxy_updater_cron(proxy_service.clone()).await;
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
    let chain_to_rpc = evm_rpc_service
        .fetch_rpcs()
        .await
        .map_err(|err| log::error!("failed to fetch rpcs from chainlist: {err}"));
    let Ok(chain_to_rpc) = chain_to_rpc else {
        return;
    };

    for chain_id in &supported_chain_ids {
        let Some(rpcs) = chain_to_rpc.get(chain_id) else {
            log::warn!("no rpc was found for {chain_id}");
            continue;
        };

        log::debug!("rpc length for {chain_id}: {}", rpcs.len());

        let mut rpc_to_metric: HashMap<String, RpcMetric> = HashMap::new();
        for rpc in rpcs {
            let metric = evm_rpc_service
                .rpc_health_check(
                    chain_id,
                    rpc,
                    proxy_service.read().await.get_proxy(),
                    feed_max_timeout,
                    // TODO(@kotsmile): remove hard code
                    3,
                )
                .await;

            log::debug!("rpc: {rpc} with metric: {metric:?}");
            rpc_to_metric.insert(rpc.to_owned(), metric);
        }

        let mut rpcs = Vec::from_iter(rpc_to_metric.iter());
        rpcs.sort_by(|&(_, a), &(_, b)| {
            let RpcMetric::Ok(a) = a else {
                return Ordering::Greater;
            };
            let RpcMetric::Ok(b) = b else {
                return Ordering::Less;
            };

            if a.response_time_ms > b.response_time_ms {
                Ordering::Less
            } else if a.response_time_ms < b.response_time_ms {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        let rpcs: Vec<(String, Metric)> = rpcs
            .iter()
            .filter_map(|&(rpc, metric)| {
                let RpcMetric::Ok(metric) = metric else {
                    return None;
                };

                Some((rpc.clone(), metric.clone()))
            })
            .collect();

        evm_rpc_service.set_rpcs_for_chain_id(chain_id, rpcs).await;
    }
}
