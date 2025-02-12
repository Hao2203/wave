use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub router: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut router = HashMap::new();
        router.insert("".to_string(), "127.0.0.1".to_string());
        router.insert("localhost".to_string(), "127.0.0.1".to_string());
        Self { router }
    }
}

pub fn init_config() -> anyhow::Result<Config> {
    let config: Option<Config> = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .and_then(|config| config.try_deserialize())?;

    Ok(config.unwrap_or_default())
}
