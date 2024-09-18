use super::*;
use anyhow::Result;
use derive_more::{AsRef, Display};
pub use file::*;
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::path::PathBuf;
use text::Text;
use ulid::Ulid;

pub mod client;
pub mod file;
pub mod text;

pub trait RecordStore {
    fn list(&self) -> impl Future<Output = Result<impl Stream<Item = Result<Record>>>> + Send;
}

pub trait RecordStoreMut: RecordStore {
    fn insert_file(&self, file: &FileRef) -> impl Future<Output = Result<()>>;

    fn insert_text(&self, text: &Text) -> impl Future<Output = Result<()>>;
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Display, AsRef)]
#[serde(transparent)]
pub struct RecordId(Ulid);

impl RecordId {
    pub fn usize(&self) -> usize {
        self.0 .0 as usize
    }
    pub fn u128(&self) -> u128 {
        self.0 .0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    id: RecordId,
    content: RecordContent,
}

impl Record {
    pub fn new(content: RecordContent) -> Self {
        let id = RecordId(Ulid::new());
        Self { id, content }
    }

    pub fn id(&self) -> &RecordId {
        &self.id
    }

    pub fn content(&self) -> &RecordContent {
        &self.content
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordContent {
    Text(Text),
    File(FileRef),
}
