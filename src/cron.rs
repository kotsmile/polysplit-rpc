use std::sync::Arc;

use anyhow::Result;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    libs::{proxy_updater::ProxyUpdaterLib, rpc_feed::RpcFeedLib},
    repo::config::ConfigRepo,
};

pub async fn run_crons(
    rpc_feed_lib: Arc<RpcFeedLib>,
    proxy_updater_lib: Arc<ProxyUpdaterLib>,
    _config_repo: ConfigRepo,
) -> Result<()> {
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

    {
        let proxy_updater_lib = proxy_updater_lib.clone();
        sched
            .add(Job::new_async("0 */1 * * * *", move |_uuid, mut _l| {
                let proxy_updater_lib = proxy_updater_lib.clone();
                Box::pin(async move {
                    log::info!("start rpc feed cron");
                    proxy_updater_lib.proxy_updater_cron().await;
                })
            })?)
            .await?;
    }

    sched.start().await?;

    Ok(())
}
