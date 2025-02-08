use config::ConfigError;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::warn;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub router: HashMap<String, String>,
}

pub fn init_config() -> anyhow::Result<Config> {
    let config = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build();

    let config = match config {
        Ok(config) => config.try_deserialize()?,
        Err(ConfigError::NotFound(e)) => {
            warn!("config not found: {}", e);
            Config {
                router: HashMap::new(),
            }
        }
        Err(e) => return Err(e.into()),
    };

    Ok(config)
}
