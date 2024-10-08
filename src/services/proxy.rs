use anyhow::{anyhow, bail, Context, Result};

use crate::{client::proxyseller::ProxysellerClient, models::proxy::ProxyConfig};

#[derive(PartialEq, Eq)]
enum ProxyServiceState {
    NotInitiated,
    Initiated,
    Other,
}

pub struct ProxyService {
    proxy_client: Box<ProxysellerClient>,
    proxy_id: usize,
    proxies: Vec<ProxyConfig>,
    state: ProxyServiceState,
}

impl ProxyService {
    pub fn new(proxy_client: Box<ProxysellerClient>) -> Self {
        Self {
            proxy_client,
            proxy_id: 0,
            proxies: Vec::new(),
            state: ProxyServiceState::NotInitiated,
        }
    }

    pub fn get_proxy(&self) -> Option<&ProxyConfig> {
        self.proxies.get(self.proxy_id)
    }

    pub async fn rotate_proxy(&mut self) -> Result<()> {
        let length = self.proxies.len();
        if length == 0 {
            bail!("proxies length is zero");
        }

        let inital_proxy_id = self.proxy_id;

        loop {
            match self.state {
                ProxyServiceState::NotInitiated => bail!("firstly call init_proxies() function"),
                ProxyServiceState::Initiated => self.state = ProxyServiceState::Other,
                ProxyServiceState::Other => {
                    self.proxy_id = (self.proxy_id + 1) % length;
                    if inital_proxy_id == self.proxy_id {
                        bail!("failed to find good proxy")
                    }
                }
            }

            let proxy_config = self.get_proxy().ok_or(anyhow!("failed to get proxy"))?;
            let response = self
                .proxy_client
                .check_proxy(&proxy_config)
                .await
                .context("failed to check proxy")?;

            if response {
                break;
            }
        }

        Ok(())
    }

    pub async fn init_proxies(&mut self) -> Result<()> {
        self.proxies = self
            .proxy_client
            .fetch_proxies()
            .await
            .context("failed to fetch proxies")?;

        self.proxy_id = 0;
        self.state = ProxyServiceState::Initiated;

        Ok(())
    }
}
