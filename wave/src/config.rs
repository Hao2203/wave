use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub router: HashMap<String, String>,
    pub proxy: ProxyConfig,
    pub server: ServerConfig,
}

impl Default for Config {
    fn default() -> Self {
        let mut router = HashMap::new();
        router.insert("".to_string(), "127.0.0.1".to_string());
        router.insert("localhost".to_string(), "127.0.0.1".to_string());

        let proxy = ProxyConfig::default();

        let server = ServerConfig::default();

        Self {
            router,
            proxy,
            server,
        }
    }
}

pub fn init_config() -> anyhow::Result<Option<Config>> {
    let config: Option<Config> = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .and_then(|config| config.try_deserialize())?;

    Ok(config)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProxyConfig {
    pub socks5: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            socks5: "127.0.0.1:8182".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub address: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:0".to_string(),
        }
    }
}
