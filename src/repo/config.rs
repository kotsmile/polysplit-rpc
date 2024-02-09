use std::time::Duration;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct ConfigRepo {
    pub port: i32,
    pub proxyseller_api_key: String,
    pub supported_chain_ids: Vec<String>,
    pub feed_max_timeout: Duration,
    pub secret_key: String,
    pub frontend_url_sign_in: String,
    pub frontend_url_profile: String,
    pub database_url: String,
    pub rocket_oauth: String,
}

fn get_env(name: &str) -> Result<String> {
    std::env::var(name).context(format!("failed to access \"{name}\" var"))
}

impl ConfigRepo {
    pub fn new() -> Result<Self> {
        let port = get_env("PORT")?
            .parse::<i32>()
            .context("failed to parse port")?;
        let proxyseller_api_key = get_env("PROXYSELLER_API_KEY")?;
        let supported_chain_ids: Vec<String> = get_env("SUPPORTED_CHAIN_IDS")?
            .split(',')
            .map(|v| v.to_owned())
            .collect();
        let feed_max_timeout = get_env("FEED_MAX_TIMEOUT_MS")?
            .parse::<u32>()
            .context("failed to parse feed max timeout")
            .map(|val| Duration::new(0, val * 1_000_000))?;

        let frontend_url_sign_in = get_env("FRONTEND_URL_SIGN_IN")?;
        let frontend_url_profile = get_env("FRONTEND_URL_PROFILE")?;
        let secret_key = get_env("SECRET_KEY")?;
        let database_url = get_env("DATABASE_URL")?;

        // oauth
        let google_client_id = get_env("GOOGLE_CLIENT_ID")?;
        let google_client_secret = get_env("GOOGLE_CLIENT_SECRET")?;
        let google_redirect_uri = get_env("GOOGLE_REDIRECT_URI")?;

        let rocket_oauth = format!(
            r#"
            {{
                google =  {{
                    provider = "Google",
                    client_id = "{google_client_id}",
                    client_secret = "{google_client_secret}",
                    redirect_uri = "{google_redirect_uri}"
                }}
            }}"#
        );

        Ok(Self {
            port,
            proxyseller_api_key,
            supported_chain_ids,
            frontend_url_sign_in,
            frontend_url_profile,
            secret_key,
            database_url,
            feed_max_timeout,
            rocket_oauth,
        })
    }
}
