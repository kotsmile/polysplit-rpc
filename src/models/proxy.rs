use anyhow::{anyhow, Result};
use reqwest::Proxy;

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: String,
}

impl ProxyConfig {
    pub fn to_proxy(&self) -> Result<Proxy> {
        Ok(Proxy::http(format!("http://{}:{}", self.host, self.port))
            .map_err(|err| anyhow!("failed to build proxy: {err}"))?
            .basic_auth(&self.username, &self.password))
    }
}
