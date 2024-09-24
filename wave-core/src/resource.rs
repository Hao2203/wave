use crate::author::Author;
use anyhow::Result;
use bytes::Bytes;
use iroh::{
    blobs::Hash,
    client::{docs, RpcClient},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncRead;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resource {
    pub hash: Hash,
    pub size: usize,
}

pub enum MakeResourceKind {
    File(PathBuf),
    Bytes(Bytes),
    Reader(Box<dyn AsyncRead + Send + Unpin + 'static>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Record<K, T> {
    pub id: K,
    pub data: T,
    pub author: Author,
    pub timestamp: u64,
}

impl<K, V> Record<K, V> {
    pub fn new(id: K, data: V, author: Author, timestamp: u64) -> Self {
        Self {
            id,
            data,
            author,
            timestamp,
        }
    }
}

impl<K, V> Record<K, V>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    pub async fn from_entry(client: impl Into<&RpcClient>, entry: docs::Entry) -> Result<Self> {
        let author = Author::new(entry.author());
        let data = entry.content_bytes(client).await?;
        let data = rmp_serde::from_slice(&data)?;
        let timestamp = entry.timestamp();
        Ok(Record::new(
            rmp_serde::from_slice(entry.key())?,
            data,
            author,
            timestamp,
        ))
    }
}
