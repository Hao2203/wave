use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

pub mod node;

pub trait Entity {
    type Id: AsRef<[u8; 32]>;
    type Data: Serialize + DeserializeOwned + Send + Sync;
}

#[async_trait::async_trait]
pub trait BlobStore {
    async fn insert(&self, data: &[u8]) -> Result<[u8; 32]>;

    async fn read(&self, id: &[u8; 32]) -> Result<Vec<u8>>;
}
