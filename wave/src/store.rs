use iroh::SecretKey;
use redb::{Database, TableDefinition};
use std::path::Path;

use crate::config::Config;

pub struct Store {
    database: Database,
}

impl Store {
    const METADATA_TABLE: TableDefinition<'_, &str, &[u8]> = TableDefinition::new("metadata");
    const SECRET_KEY: &str = "secret_key";
    const CONFIG: &str = "config";

    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let database = Database::open(path)?;
        let tx = database.begin_write()?;
        {
            let _ = tx.open_table(Self::METADATA_TABLE)?;
        }
        tx.commit()?;
        Ok(Self { database })
    }

    pub fn get_secret_key(&self) -> anyhow::Result<Option<SecretKey>> {
        let reader = self.database.begin_read()?;
        let table = reader.open_table(Self::METADATA_TABLE)?;

        let secret_key = table
            .get(Self::SECRET_KEY)?
            .map(|v| SecretKey::try_from(v.value()))
            .transpose()?;

        Ok(secret_key)
    }

    #[allow(clippy::let_and_return)]
    pub fn put_secret_key(&self, secret_key: &SecretKey) -> anyhow::Result<Option<SecretKey>> {
        let writer = self.database.begin_write()?;
        let secret_key = {
            let mut table = writer.open_table(Self::METADATA_TABLE)?;

            let secret_key = table
                .insert(Self::SECRET_KEY, secret_key.to_bytes().as_slice())?
                .map(|v| SecretKey::try_from(v.value()))
                .transpose()?;
            secret_key
        };

        writer.commit()?;

        Ok(secret_key)
    }

    pub fn get_config(&self) -> anyhow::Result<Option<Config>> {
        let reader = self.database.begin_read()?;
        let table = reader.open_table(Self::METADATA_TABLE)?;

        let config = table
            .get(Self::CONFIG)?
            .map(|v| Self::bytes_to_config(v.value()))
            .transpose()?;

        Ok(config)
    }

    #[allow(clippy::let_and_return)]
    pub fn put_config(&self, config: &Config) -> anyhow::Result<Option<Config>> {
        let writer = self.database.begin_write()?;

        let config = {
            let mut table = writer.open_table(Self::METADATA_TABLE)?;
            let config = table
                .insert(Self::CONFIG, Self::config_to_bytes(config)?.as_slice())?
                .map(|v| Self::bytes_to_config(v.value()))
                .transpose()?;
            config
        };
        writer.commit()?;

        Ok(config)
    }

    fn config_to_bytes(config: &Config) -> anyhow::Result<Vec<u8>> {
        Ok(rmp_serde::to_vec(config)?)
    }

    fn bytes_to_config(bytes: &[u8]) -> anyhow::Result<Config> {
        Ok(rmp_serde::from_slice(bytes)?)
    }
}
