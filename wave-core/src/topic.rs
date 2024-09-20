use async_stream::stream;
use author::Author;
use futures::{stream::BoxStream, StreamExt};
use iroh::client::{
    blobs,
    docs::{self, LiveEvent},
};
use ulid::Ulid;

use super::*;

#[async_trait::async_trait]
pub trait Topic<T> {
    async fn publish(&self, author: &Author, message: &T) -> Result<()>;

    async fn subscribe(&self) -> Result<BoxStream<Result<Message<T>>>>;
}

pub struct Client<'a> {
    doc: docs::Doc,
    client: &'a blobs::Client,
}

#[async_trait::async_trait]
impl<'a, T> Topic<T> for Client<'a>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'a,
{
    async fn publish(&self, author: &Author, message: &T) -> Result<()> {
        let author_id = author.as_bytes().into();
        let key = Ulid::new().to_string();
        let value = rmp_serde::to_vec(&message)?;
        self.doc.set_bytes(author_id, key, value).await?;
        Ok(())
    }

    async fn subscribe(&self) -> Result<BoxStream<Result<Message<T>>>> {
        let mut stream = self.doc.subscribe().await?;
        let stream = stream! {
            while let Some(event) = stream.next().await {
                let res = self.event_handle(event?).await?;
                yield Ok(res)
            }
        };
        let stream = Box::pin(stream);
        Ok(stream)
    }
}

impl Client<'_> {
    async fn event_handle<T: DeserializeOwned>(&self, event: LiveEvent) -> Result<Message<T>> {
        match event {
            LiveEvent::InsertLocal { entry } | LiveEvent::InsertRemote { entry, .. } => {
                let author = Author::new(entry.author());
                let data = self.client.read_to_bytes(entry.content_hash()).await?;
                let data = rmp_serde::from_slice(&data)?;
                let timestamp = entry.timestamp();
                Ok(Message::new(data, author, timestamp))
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Message<T> {
    id: Ulid,
    data: T,
    author: Author,
    timestamp: u64,
}

impl<T> Message<T> {
    pub fn new(data: T, author: Author, timestamp: u64) -> Self {
        let id = Ulid::new();
        Self {
            id,
            data,
            author,
            timestamp,
        }
    }

    pub fn id(&self) -> u128 {
        self.id.into()
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn into_data(self) -> T {
        self.data
    }

    pub fn author(&self) -> &Author {
        &self.author
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}
