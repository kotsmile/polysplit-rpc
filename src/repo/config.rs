use anyhow::{anyhow, Result};

pub struct ConfigRepo {
    pub port: i32,
    pub proxyseller_api_key: String,
}

impl ConfigRepo {
    pub fn new() -> Result<Self> {
        let port = std::env::var("PORT")
            .map_err(|err| anyhow!("failed to access \"JWT_SECRET_KEY\" var: {err}"))
            .and_then(|p| {
                p.parse::<i32>()
                    .map_err(|err| anyhow!("failed to parse port: {err}"))
            })?;
        let proxyseller_api_key = std::env::var("PROXYSELLER_API_KEY")
            .map_err(|err| anyhow!("failed to access \"JWT_SECRET_KEY\" var: {err}"))?;

        Ok(Self {
            port,
            proxyseller_api_key,
        })
    }
}
