use crate::author::Author;
use anyhow::Result;
use bytes::Bytes;
use iroh::docs::store::Query;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

pub trait KVStore {
    fn insert(
        &self,
        key: impl Into<Bytes> + Send,
        value: impl Serialize + Send,
    ) -> impl Future<Output = Result<()>> + Send;

    fn get<T>(
        &self,
        key: impl Into<Bytes> + Send,
    ) -> impl Future<Output = Result<Option<T>>> + Send
    where
        T: DeserializeOwned;
}

pub trait MakeStore {
    type Id: AsRef<[u8; 32]> + Send;
    type Store: KVStore + Send;

    fn make(&self, author: &Author)
        -> impl Future<Output = Result<(Self::Id, Self::Store)>> + Send;

    fn get_store(
        &self,
        author: &Author,
        id: impl AsRef<[u8; 32]> + Send,
    ) -> impl Future<Output = Result<Option<Self::Store>>> + Send;
}

pub struct DocStore {
    doc: iroh::client::docs::Doc,
    author_id: iroh::docs::AuthorId,
}

impl DocStore {
    pub fn new(doc: iroh::client::docs::Doc, author_id: iroh::docs::AuthorId) -> Self {
        Self { doc, author_id }
    }
}

impl KVStore for DocStore {
    async fn get<T>(&self, key: impl Into<Bytes>) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let res = self.doc.get_one(Query::key_exact(key.into())).await?;

        if let Some(entry) = res {
            let res = entry.content_bytes(&self.doc).await?;
            Ok(Some(rmp_serde::from_slice(&res)?))
        } else {
            Ok(None)
        }
    }

    async fn insert(&self, key: impl Into<Bytes>, value: impl Serialize) -> Result<()> {
        let _res = self
            .doc
            .set_bytes(self.author_id, key, rmp_serde::to_vec(&value)?)
            .await?;
        Ok(())
    }
}
