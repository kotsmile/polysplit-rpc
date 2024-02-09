use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use rocket::tokio::{sync::RwLock, task};

use crate::{
    crons::rpc_feed_cron,
    repo::config::ConfigRepo,
    services::{
        evm_rpc::{EvmRpcService, RpcMetrics},
        group::GroupService,
        proxy::ProxyService,
    },
};

pub async fn run_tasks(
    evm_rpc_service: Arc<EvmRpcService>,
    group_service: Arc<GroupService>,
    proxy_service: Arc<RwLock<ProxyService>>,
    config_repo: ConfigRepo,
) {
    {
        log::info!("start init proxies");
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
        log::info!("start rpc feed cron");
        let evm_rpc_service = evm_rpc_service.clone();
        let proxy_service = proxy_service.clone();
        let supported_chain_ids = config_repo.supported_chain_ids.clone();
        let feed_max_timeout = config_repo.feed_max_timeout.clone();
        let group_service = group_service.clone();
        task::spawn(async move {
            rpc_feed_cron(
                evm_rpc_service,
                proxy_service,
                supported_chain_ids,
                feed_max_timeout,
                group_service,
            )
            .await;
        });
    }

    log::info!("fetching rpcs");
    let chain_to_rpc = evm_rpc_service
        .fetch_rpcs()
        .await
        .map_err(|err| log::error!("failed to fetch rpcs from chainlist: {err}"));
    let Ok(chain_to_rpc) = chain_to_rpc else {
        return;
    };

    // zero initialize states
    log::info!("initiating rpcs");
    for chain_id in &config_repo.supported_chain_ids {
        let Some(rpcs) = chain_to_rpc.get(chain_id) else {
            continue;
        };

        let mut rpcs_for_chain_id: Vec<(String, RpcMetrics)> = Vec::new();
        for rpc in rpcs {
            rpcs_for_chain_id.push((
                rpc.clone(),
                RpcMetrics {
                    response_time_ms: 0,
                },
            ));
        }

        evm_rpc_service
            .set_rpcs_for_chain_id_cache(chain_id, rpcs_for_chain_id)
            .await;
    }

    log::info!("initiating groups");
    let Ok(groups) = group_service
        .get_groups()
        .await
        .context("failed to groups")
        .map_err(|err| log::error!("{err}"))
    else {
        log::error!("failed to get groups");
        return;
    };

    // TODO: add checking for enable
    for group in &groups {
        let Ok(rpcs) = group_service
            .get_group_rpcs(&group.id)
            .await
            .map_err(|err| log::error!("{err}"))
        else {
            log::error!("failed to get rpcs for group");
            continue;
        };

        let mut chain_id_to_rpcs: HashMap<String, Vec<String>> = HashMap::new();
        for rpc in &rpcs {
            chain_id_to_rpcs
                .entry(rpc.chain_id.clone())
                .or_insert(Vec::new())
                .push(rpc.url.clone());
        }

        for (chain_id, rpcs) in chain_id_to_rpcs {
            evm_rpc_service
                .set_rpcs_for_api_key_cache(&group.api_key, &chain_id, rpcs)
                .await;
        }
    }

    log::info!("initiating chains");
    if let Err(err) = evm_rpc_service
        .init_chains(&config_repo.supported_chain_ids)
        .await
        .context("failed to init chains in evm_rpc service")
    {
        panic!("{err}");
    }

    log::info!("updating public rpcs");
    if let Err(err) = evm_rpc_service
        .update_public_rpcs()
        .await
        .context("failed to update public rpcs in evm_rpc service")
    {
        log::warn!("{err}");
    }
}
