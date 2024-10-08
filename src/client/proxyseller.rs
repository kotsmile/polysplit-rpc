use std::collections::HashMap;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;

use crate::models::proxy::ProxyConfig;

#[derive(Clone)]
pub struct ProxysellerOrder(pub String, pub String);

const PROXYSELLER_BASE_URL_API: &'static str = "https://proxy-seller.com/personal/api/v1";

#[derive(Deserialize)]
struct ProxysellerFetchProxiesDataElement {
    ip: String,
    port_http: i32,
    login: String,
    password: String,
    // id: String,
    // order_id: String,
    // protocol: String,
    // port_socks: i32,
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

#[derive(Clone)]
pub struct ProxysellerClient {
    api_key: String,
    orders: Vec<ProxysellerOrder>,
    timeout_ms: i32,
}

impl ProxysellerClient {
    pub fn new(
        proxyseller_api_key: String,
        orders: Vec<ProxysellerOrder>,
        timeout_ms: i32,
    ) -> Self {
        Self {
            api_key: proxyseller_api_key,
            orders,
            timeout_ms,
        }
    }

    pub async fn fetch_proxies(&self) -> Result<Vec<ProxyConfig>> {
        let mut proxy_configs: Vec<ProxyConfig> = Vec::new();
        for order in &self.orders {
            let params = [("latest", "y"), ("orderId", &order.1)];
            let response = reqwest::Client::new()
                .get(format!(
                    "{PROXYSELLER_BASE_URL_API}/{api_key}/proxy/list/{order_type}",
                    api_key = self.api_key,
                    order_type = order.0
                ))
                .query(&params)
                .send()
                .await
                .context("failed to request proxy list")?
                .json::<ProxysellerResponse<ProxysellerFetchProxiesDataWrapper>>()
                .await
                .context("failed to deserialize fetch request")?;

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

    pub async fn check_proxy(&self, proxy_config: &ProxyConfig) -> Result<bool> {
        let proxy_string = format!(
            "{username}:{password}@{host}:{port}",
            username = proxy_config.username,
            password = proxy_config.password,
            host = proxy_config.host,
            port = proxy_config.port
        );
        let params = [("proxy", proxy_string)];
        let response = reqwest::Client::new()
            .get(format!(
                "{PROXYSELLER_BASE_URL_API}/{api_key}/tools/proxy/check",
                api_key = self.api_key
            ))
            .query(&params)
            .send()
            .await
            .context("failed to request check for proxy")?
            .json::<ProxysellerResponse<ProxysellerCheckProxyData>>()
            .await
            .context("failed to deserialize check request")?;

        if response.status != "success".to_string() {
            bail!("check response status not equal success");
        }

        Ok(response.data.valid && response.data.time < self.timeout_ms)
    }
}
