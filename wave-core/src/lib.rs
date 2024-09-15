use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

pub mod node;

pub trait Entity {
    type Id: AsRef<[u8; 32]>;
    type Data: Serialize + DeserializeOwned + Send + Sync;
}

pub trait EntityStore<E: Entity> {
    fn get_all(&self) -> impl Future<Output = Result<Vec<E::Id>>> + Send;

    fn get(&self, id: &E::Id) -> impl Future<Output = Result<Option<E::Data>>> + Send;

    fn insert(&self, data: &E::Data) -> impl Future<Output = Result<E::Id>> + Send;

    fn delete(&self, id: &E::Id) -> impl Future<Output = Result<()>> + Send;

    fn check(&self, id: &E::Id) -> impl Future<Output = Result<bool>> + Send;
}

pub trait EntityList<E: Entity> {
    type ListId: AsRef<[u8; 32]>;

    fn create(&self) -> impl Future<Output = Result<Self::ListId>> + Send;

    fn push(
        &self,
        list_id: &Self::ListId,
        data: E::Data,
    ) -> impl Future<Output = Result<E::Id>> + Send;

    fn list(&self, list_id: &Self::ListId) -> impl Future<Output = Result<Vec<E::Id>>> + Send;
}
