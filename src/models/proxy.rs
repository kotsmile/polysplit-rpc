#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: String,
}
