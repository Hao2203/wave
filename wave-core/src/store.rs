use super::*;
use anyhow::Result;
use author::Author;
use bytes::Bytes;
use iroh::docs::store::Query;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

pub trait MakeStore {
    type Id: AsRef<[u8; 32]> + Send;
    type Store<'a>: KVStore + Send + 'a;

    fn make<'a>(
        &self,
        author: &'a Author,
    ) -> impl Future<Output = Result<(Self::Id, Self::Store<'a>)>> + Send;

    fn get_store<'a>(
        &self,
        author: &'a Author,
        id: impl AsRef<[u8; 32]> + Send,
    ) -> impl Future<Output = Result<Option<Self::Store<'a>>>> + Send;
}

impl MakeStore for WaveClient {
    type Store<'a> = DocStore<'a>;
    type Id = NamespaceId;

    async fn make<'a>(&self, author: &'a Author) -> Result<(Self::Id, Self::Store<'a>)> {
        let doc = self.node().docs().create().await?;
        Ok((doc.id(), DocStore::new(doc, author.id())))
    }

    async fn get_store<'a>(
        &self,
        author: &'a Author,
        id: impl AsRef<[u8; 32]>,
    ) -> Result<Option<Self::Store<'a>>> {
        let doc = self.node().docs().open(id.as_ref().into()).await?;
        Ok(doc.map(|doc| DocStore::new(doc, author.id())))
    }
}

impl<T: iroh::blobs::store::Store> MakeStore for iroh::node::Node<T> {
    type Id = NamespaceId;
    type Store<'a> = DocStore<'a>;

    async fn get_store<'a>(
        &self,
        author: &'a Author,
        id: impl AsRef<[u8; 32]> + Send,
    ) -> Result<Option<Self::Store<'a>>> {
        let doc = self.docs().open(id.as_ref().into()).await?;
        Ok(doc.map(|doc| DocStore::new(doc, author.id())))
    }

    async fn make<'a>(&self, author: &'a Author) -> Result<(Self::Id, Self::Store<'a>)> {
        let doc = self.docs().create().await?;
        Ok((doc.id(), DocStore::new(doc, author.id())))
    }
}

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

pub struct DocStore<'a> {
    doc: iroh::client::docs::Doc,
    author_id: &'a iroh::docs::AuthorId,
}

impl<'a> DocStore<'a> {
    pub fn new(doc: iroh::client::docs::Doc, author_id: &'a iroh::docs::AuthorId) -> Self {
        Self { doc, author_id }
    }
}

impl<'a> KVStore for DocStore<'a> {
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
            .set_bytes(*self.author_id, key, rmp_serde::to_vec(&value)?)
            .await?;
        Ok(())
    }
}
