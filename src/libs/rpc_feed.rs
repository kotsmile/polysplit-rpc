use std::{cmp::Ordering, collections::HashMap, sync::Arc, time::Duration};

use rocket::tokio::sync::RwLock;

use crate::services::{
    evm_rpc::{EvmRpcService, Metric, RpcMetric},
    proxy::ProxyService,
};

#[derive(Clone)]
pub struct RpcFeedLib {
    evm_rpc_service: Arc<EvmRpcService>,
    proxy_service: Arc<RwLock<ProxyService>>,
    supported_chain_ids: Vec<String>,
    feed_max_timeout: Duration,
}

impl RpcFeedLib {
    pub fn new(
        evm_rpc_service: Arc<EvmRpcService>,
        proxy_service: Arc<RwLock<ProxyService>>,
        supported_chain_ids: Vec<String>,
        feed_max_timeout: Duration,
    ) -> Self {
        Self {
            evm_rpc_service,
            proxy_service,
            supported_chain_ids,
            feed_max_timeout,
        }
    }

    pub async fn rpc_feed_cron(&self) {
        let chain_to_rpc = self
            .evm_rpc_service
            .fetch_rpcs()
            .await
            .map_err(|err| log::error!("failed to fetch rpcs from chainlist: {err}"));
        let Ok(chain_to_rpc) = chain_to_rpc else {
            return;
        };

        for chain_id in &self.supported_chain_ids {
            let Some(rpcs) = chain_to_rpc.get(chain_id) else {
                log::warn!("no rpc was found for {chain_id}");
                continue;
            };

            log::debug!("rpc length for {chain_id}: {}", rpcs.len());

            let mut rpc_to_metric: HashMap<String, RpcMetric> = HashMap::new();
            for rpc in rpcs {
                let metric = self
                    .evm_rpc_service
                    .rpc_health_check(
                        chain_id,
                        rpc,
                        self.proxy_service.read().await.get_proxy(),
                        self.feed_max_timeout,
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

            self.evm_rpc_service
                .set_rpcs_for_chain_id(chain_id, rpcs)
                .await;
        }
    }
}
