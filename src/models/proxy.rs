use anyhow::{Context, Result};
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
            .context("failed to build proxy")?
            .basic_auth(&self.username, &self.password))
    }
}
