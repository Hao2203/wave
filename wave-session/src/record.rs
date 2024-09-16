use super::*;
pub use file::*;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::path::PathBuf;
use text::Text;
use zerocopy::AsBytes;

pub mod file;
pub mod text;

#[derive(Debug)]
pub struct Record {
    pub content: Content,
}

#[derive(Debug)]
pub enum Content {
    Text(Text),
    File(File),
}
