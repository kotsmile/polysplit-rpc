use anyhow::{anyhow, bail, Context, Result};
use rocket::tokio::sync::RwLock;

use crate::{client::proxyseller::ProxysellerClient, models::proxy::ProxyConfig};

#[derive(PartialEq, Eq, Clone)]
enum ProxyServiceState {
    NotInitiated,
    Initiated,
    Other,
}

pub struct ProxyService {
    proxy_client: Box<ProxysellerClient>,
    proxy_id: RwLock<usize>,
    proxies: RwLock<Vec<ProxyConfig>>,
    state: RwLock<ProxyServiceState>,
}

impl ProxyService {
    pub fn new(proxy_client: Box<ProxysellerClient>) -> Self {
        Self {
            proxy_client,
            proxy_id: RwLock::new(0),
            proxies: RwLock::new(Vec::new()),
            state: RwLock::new(ProxyServiceState::NotInitiated),
        }
    }

    pub async fn get_proxy(&self) -> Option<ProxyConfig> {
        self.proxies
            .read()
            .await
            .get(self.proxy_id.read().await.clone())
            .map(|v| v.clone())
    }

    pub async fn rotate_proxy(&self) -> Result<()> {
        let length = self.proxies.read().await.len();
        if length == 0 {
            bail!("proxies length is zero");
        }

        let initial_proxy_id;
        {
            initial_proxy_id = self.proxy_id.read().await.clone();
        }

        loop {
            let state;
            {
                state = self.state.read().await.clone();
            }

            match state {
                ProxyServiceState::NotInitiated => bail!("firstly call init_proxies() function"),
                ProxyServiceState::Initiated => {
                    *self.state.write().await = ProxyServiceState::Other
                }
                ProxyServiceState::Other => {
                    let mut proxy_id = self.proxy_id.write().await;
                    *proxy_id = (*proxy_id + 1) & length;
                    if initial_proxy_id == *proxy_id {
                        bail!("failed to find good proxy")
                    }
                }
            }

            let proxy_config = self
                .get_proxy()
                .await
                .ok_or(anyhow!("failed to get proxy"))?;
            let response = self
                .proxy_client
                .check_proxy(&proxy_config)
                .await
                .context("failed to check proxy in proxy client")?;

            if response {
                break;
            }
        }

        Ok(())
    }

    pub async fn init_proxies(&self) -> Result<()> {
        *self.proxies.write().await = self
            .proxy_client
            .fetch_proxies()
            .await
            .context("failed to fetch proxies from proxy client")?;

        *self.proxy_id.write().await = 0;
        *self.state.write().await = ProxyServiceState::Initiated;

        Ok(())
    }
}
