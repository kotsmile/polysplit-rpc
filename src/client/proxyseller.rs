use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::{models::proxy::ProxyConfig, traits::ProxyClient};

pub struct ProxysellerOrder(pub String, pub String);

pub struct ProxysellerClient {
    api_client: Client,
    api_key: String,
    orders: Vec<ProxysellerOrder>,
    timeout_ms: i32,
}

const PROXYSELLER_BASE_URL_API: &'static str = "https://proxy-seller.com/personal/api/v1";

#[derive(Deserialize)]
struct ProxysellerFetchProxiesDataElement {
    // id: String,
    // order_id: String,
    ip: String,
    // protocol: String,
    port_http: i32,
    // port_socks: i32,
    login: String,
    password: String,
    // auth_ip: String,
    // country: String,
    // status: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ProxysellerFetchProxiesData {
    Array(Vec<ProxysellerFetchProxiesDataElement>),
    Record(HashMap<String, ProxysellerFetchProxiesDataElement>),
}

#[derive(Deserialize)]
struct ProxysellerFetchProxiesDataWrapper {
    items: ProxysellerFetchProxiesData,
}

#[derive(Deserialize)]
struct ProxysellerCheckProxyData {
    valid: bool,
    time: i32,
}

#[derive(Deserialize)]
struct ProxysellerResponse<D> {
    status: String,
    data: D,
}

impl ProxysellerClient {
    pub fn new(
        proxyseller_api_key: String,
        orders: Vec<ProxysellerOrder>,
        timeout_ms: i32,
    ) -> Self {
        Self {
            api_client: reqwest::Client::new(),
            api_key: proxyseller_api_key,
            orders,
            timeout_ms,
        }
    }
}

#[async_trait]
impl ProxyClient for ProxysellerClient {
    async fn fetch_proxies(&self) -> Result<Vec<ProxyConfig>> {
        let mut proxy_configs: Vec<ProxyConfig> = Vec::new();
        for order in &self.orders {
            let params = [("latest", "y"), ("orderId", &order.1)];
            let response = self
                .api_client
                .get(format!(
                    "{PROXYSELLER_BASE_URL_API}/{api_key}/proxy/list/{order_type}",
                    api_key = self.api_key,
                    order_type = order.0
                ))
                .query(&params)
                .send()
                .await
                .map_err(|err| anyhow!("failed to request proxy list: {err}"))?
                .json::<ProxysellerResponse<ProxysellerFetchProxiesDataWrapper>>()
                .await
                .map_err(|err| anyhow!("failed to deserialize fetch request: {err}"))?;

            match response.data.items {
                ProxysellerFetchProxiesData::Array(arr) => {
                    for el in arr {
                        proxy_configs.push(ProxyConfig {
                            host: el.ip,
                            port: el.port_http,
                            username: el.login,
                            password: el.password,
                        })
                    }
                }
                ProxysellerFetchProxiesData::Record(map) => {
                    for (_, el) in map {
                        proxy_configs.push(ProxyConfig {
                            host: el.ip,
                            port: el.port_http,
                            username: el.login,
                            password: el.password,
                        })
                    }
                }
            }
        }

        Ok(proxy_configs)
    }

    async fn check_proxy(&self, proxy_config: &ProxyConfig) -> Result<bool> {
        let proxy_string = format!(
            "{username}:{password}@{host}:{port}",
            username = proxy_config.username,
            password = proxy_config.password,
            host = proxy_config.host,
            port = proxy_config.port
        );
        let params = [("proxy", proxy_string)];
        let response = self
            .api_client
            .get(format!(
                "{PROXYSELLER_BASE_URL_API}/{api_key}/tools/proxy/check",
                api_key = self.api_key
            ))
            .query(&params)
            .send()
            .await
            .map_err(|err| anyhow!("failed to request check for proxy: {err}"))?
            .json::<ProxysellerResponse<ProxysellerCheckProxyData>>()
            .await
            .map_err(|err| anyhow!("failed to deserialize check request: {err}"))?;

        if response.status != "success".to_string() {
            bail!("check response status not equal success")
        }

        Ok(response.data.valid && response.data.time < self.timeout_ms)
    }
}
