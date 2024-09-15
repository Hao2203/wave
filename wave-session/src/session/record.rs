use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use zerocopy::AsBytes;

use super::*;

#[derive(Debug, AsRef, AsBytes, Serialize, Deserialize)]
#[as_ref(forward)]
#[repr(C)]
#[serde(transparent)]
pub struct RecordsId(#[serde(with = "serde_bytes")] [u8; 32]);

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Record {
    pub id: RecordsId,
    pub content: Content,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Content {
    Text(String),
    Binary(ByteBuf),
}
