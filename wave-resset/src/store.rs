use crate::asset::Assets;
use crate::file::FileRef;
use anyhow::Result;
use iroh::client::{blobs, docs};
use iroh::docs::NamespaceId;
use std::future::Future;

pub trait AssetReadableStore {
    type Key: Send;

    fn keys(&self) -> impl Future<Output = Result<Vec<Self::Key>>> + Send;

    fn get(&self, key: &Self::Key) -> impl Future<Output = Result<Assets>> + Send;
}

pub trait AssetWritableStore: AssetReadableStore {
    fn create(&self, assets: &Assets) -> impl Future<Output = Result<Self::Key>> + Send;

    fn insert_file(
        &self,
        key: &Self::Key,
        file: &FileRef,
    ) -> impl Future<Output = Result<()>> + Send;

    fn remove(&self, key: &Self::Key) -> impl Future<Output = Result<()>> + Send;
}

pub struct AssectStore<'a> {
    pub blob_clent: &'a blobs::Client,
    pub doc_clent: &'a docs::Client,
}

impl AssetReadableStore for AssectStore<'_> {
    type Key = NamespaceId;

    async fn get(&self, key: &Self::Key) -> Result<Assets> {
        todo!()
    }

    async fn keys(&self) -> Result<Vec<Self::Key>> {
        todo!()
    }
}
