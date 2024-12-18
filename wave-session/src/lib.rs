use async_trait::async_trait;
use bytes::Bytes;
use chat::ChatId;
use futures_lite::stream::Boxed as BoxStream;

pub mod chat;
pub mod error;
pub mod message;

#[cfg(test)]
mod test;

#[async_trait]
pub trait ChatTopic {
    type Error;
    async fn subscribe(&self, chat_id: ChatId) -> Result<BoxStream<Bytes>, Self::Error>;

    async fn publish(&self, chat_id: ChatId, message: Bytes) -> Result<(), Self::Error>;
}
