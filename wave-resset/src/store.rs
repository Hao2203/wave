use anyhow::Result;
use std::{future::Future, path::Path};

pub mod client;

pub trait FileContentStore {
    type Hash: AsRef<[u8]> + Send + Sync;

    fn get_size(&self, hash: &Self::Hash) -> impl Future<Output = Result<Option<usize>>> + Send;

    fn insert(&self, file_path: &Path) -> impl Future<Output = Result<Self::Hash>> + Send;
}
