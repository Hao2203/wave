use super::*;
use anyhow::Result;
use futures::stream::BoxStream;
use iroh::client::docs;

#[async_trait::async_trait]
pub trait MessageStore<T> {
    async fn push(&self, message: &T) -> Result<Message<T>>;

    async fn list(&self) -> Result<BoxStream<Message<T>>>;

    async fn list_by_author(&self, author: &Author) -> Result<BoxStream<Message<T>>>;

    async fn get(&self, id: u128) -> Result<Option<Message<T>>>;

    async fn subscribe(&self) -> Result<BoxStream<Message<T>>>;
}

#[derive(Clone, Copy)]
pub struct Store<'a> {
    doc: &'a docs::Doc,
    author: &'a Author,
}

#[async_trait::async_trait]
impl<T> MessageStore<T> for Store<'_>
where
    T: Serialize + DeserializeOwned + MakeResource + Send + Sync,
{
    async fn push(&self, message: &T) -> Result<Message<T>> {
        todo!()
    }

    async fn list(&self) -> Result<BoxStream<Message<T>>> {
        todo!()
    }

    async fn list_by_author(&self, author: &Author) -> Result<BoxStream<Message<T>>> {
        todo!()
    }

    async fn get(&self, id: u128) -> Result<Option<Message<T>>> {
        todo!()
    }

    async fn subscribe(&self) -> Result<BoxStream<Message<T>>> {
        todo!()
    }
}
