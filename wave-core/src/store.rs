use crate::author::Author;
use anyhow::Result;
use bytes::Bytes;
use iroh::docs::{store::Query, NamespaceId};
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

pub trait KVStore {
    type Id: AsRef<[u8; 32]>;

    fn id(&self) -> Self::Id;

    fn insert(
        &self,
        key: impl Into<Bytes> + Send,
        value: &(impl Serialize + Sync),
    ) -> impl Future<Output = Result<()>> + Send;

    fn get<T>(
        &self,
        key: impl Into<Bytes> + Send,
    ) -> impl Future<Output = Result<Option<T>>> + Send
    where
        T: DeserializeOwned;
}

pub trait MakeStore {
    fn make(&self, author: &Author) -> impl Future<Output = Result<impl KVStore + Send>> + Send;

    fn get_store(
        &self,
        author: &Author,
        id: impl AsRef<[u8]>,
    ) -> impl Future<Output = Result<Option<impl KVStore + Send>>> + Send;
}

pub struct DocStore<'a> {
    doc: &'a iroh::client::docs::Doc,
    author_id: iroh::docs::AuthorId,
}

impl<'a> KVStore for DocStore<'a> {
    type Id = NamespaceId;

    fn id(&self) -> Self::Id {
        self.doc.id()
    }

    async fn get<T>(&self, key: impl Into<Bytes> + Send) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let res = self.doc.get_one(Query::key_exact(key.into())).await?;

        if let Some(entry) = res {
            let res = entry.content_bytes(self.doc).await?;
            Ok(Some(rmp_serde::from_slice(&res)?))
        } else {
            Ok(None)
        }
    }

    async fn insert(
        &self,
        key: impl Into<Bytes> + Send,
        value: &(impl Serialize + Sync),
    ) -> Result<()> {
        let _res = self
            .doc
            .set_bytes(self.author_id, key, rmp_serde::to_vec(value)?)
            .await?;
        Ok(())
    }
}
