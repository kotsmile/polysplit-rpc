use std::time::Duration;

use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct ConfigRepo {
    pub port: i32,
    pub proxyseller_api_key: String,
    pub supported_chain_ids: Vec<String>,
    pub feed_max_timeout: Duration,
}

fn get_env(name: &str) -> Result<String> {
    std::env::var(name).map_err(|err| anyhow!("failed to access \"{name}\" var: {err}"))
}

impl ConfigRepo {
    pub fn new() -> Result<Self> {
        let port = get_env("PORT")?
            .parse::<i32>()
            .map_err(|err| anyhow!("failed to parse port: {err}"))?;
        let proxyseller_api_key = get_env("PROXYSELLER_API_KEY")?;
        let supported_chain_ids: Vec<String> = get_env("SUPPORTED_CHAIN_IDS")?
            .split(',')
            .map(|v| v.to_owned())
            .collect();
        let feed_max_timeout = get_env("FEED_MAX_TIMEOUT_MS")?
            .parse::<u32>()
            .map_err(|err| anyhow!("failed to parse feed max timeout: {err}"))
            .map(|val| Duration::new(0, val * 1_000_000))?;

        Ok(Self {
            port,
            proxyseller_api_key,
            supported_chain_ids,
            feed_max_timeout,
        })
    }
}
