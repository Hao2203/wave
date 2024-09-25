use crate::author::Author;
use anyhow::Result;
use bytes::Bytes;
use iroh::{
    blobs::Hash,
    client::{docs, RpcClient},
};
use serde::{Deserialize, Serialize};
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
pub struct Record {
    pub id: Bytes,
    pub data: Bytes,
    pub author: Author,
    pub timestamp: u64,
}

impl Record {
    pub fn new(id: Bytes, data: Bytes, author: Author, timestamp: u64) -> Self {
        Self {
            id,
            data,
            author,
            timestamp,
        }
    }
}

impl Record {
    pub async fn from_entry(client: impl Into<&RpcClient>, entry: docs::Entry) -> Result<Self> {
        let author = Author::new(entry.author());
        let data = entry.content_bytes(client).await?;
        let timestamp = entry.timestamp();
        let id = Bytes::copy_from_slice(entry.key());
        Ok(Record::new(id, data, author, timestamp))
    }
}
