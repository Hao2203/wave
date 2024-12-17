use async_trait::async_trait;
use bytes::Bytes;
use futures_lite::stream::Boxed as BoxStream;

pub mod error;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub struct Session {
    pub name: String,
}

impl Session {
    pub fn new(name: String) -> Session {
        Session { name }
    }
}

#[async_trait]
pub trait Manager {
    type Error;
    type TopicId;
    async fn create(&self, name: String) -> Result<Self::TopicId, Self::Error>;

    async fn publish(&self, topic: &Self::TopicId, bytes: Bytes) -> Result<(), Self::Error>;

    async fn subscribe(&self, topic: &Self::TopicId) -> Result<BoxStream<Bytes>, Self::Error>;
}
