use std::path::PathBuf;

use bytes::Bytes;
use iroh::blobs::Hash;
use serde::{Deserialize, Serialize};
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
