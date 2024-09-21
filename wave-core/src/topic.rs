use super::*;
use async_stream::stream;
use author::Author;
use futures::stream::BoxStream;
use iroh::{
    client::{
        blobs,
        docs::{self, LiveEvent},
    },
    docs::store::Query,
};
use ulid::Ulid;

#[async_trait::async_trait]
pub trait Topic<T> {
    async fn get(&self, id: u128) -> Result<Option<Message<T>>>;

    async fn publish(&self, author: &Author, message: &T) -> Result<u128>;

    async fn subscribe(&self) -> Result<BoxStream<Result<Message<T>>>>;
}

pub struct Client<'a> {
    doc: docs::Doc,
    client: &'a blobs::Client,
}

impl<'a> Client<'a> {
    #[cfg(test)]
    pub async fn mock(node: &'a iroh::client::Iroh) -> Result<Self> {
        let doc = node.docs().create().await?;
        Ok(Self {
            doc,
            client: node.blobs(),
        })
    }
}

#[async_trait::async_trait]
impl<'a, T> Topic<T> for Client<'a>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'a,
{
    async fn get(&self, id: u128) -> Result<Option<Message<T>>> {
        let id = Ulid::from(id);
        let key = id.to_string();
        let entry = self.doc.get_one(Query::key_exact(key)).await?;
        if let Some(entry) = entry {
            let author = Author::new(entry.author());
            let data = self.client.read_to_bytes(entry.content_hash()).await?;
            let data = rmp_serde::from_slice(&data)?;
            let timestamp = entry.timestamp();
            Ok(Some(Message::new(data, author, timestamp)))
        } else {
            Ok(None)
        }
    }

    async fn publish(&self, author: &Author, message: &T) -> Result<u128> {
        let author_id = author.as_bytes().into();
        let id = Ulid::new();
        let key = id.to_string();
        let value = rmp_serde::to_vec(&message)?;
        self.doc.set_bytes(author_id, key, value).await?;
        Ok(id.0)
    }

    async fn subscribe(&self) -> Result<BoxStream<Result<Message<T>>>> {
        let stream = self.doc.subscribe().await?;
        let stream = stream! {
            for await event in stream {
                match event? {
                    LiveEvent::InsertLocal { entry } | LiveEvent::InsertRemote { entry, .. } => {
                        let author = Author::new(entry.author());
                        let data = self.client.read_to_bytes(entry.content_hash()).await?;
                        let data = rmp_serde::from_slice(&data)?;
                        let timestamp = entry.timestamp();
                        yield Ok(Message::new(data, author, timestamp))
                    }
                    _ => (),
                }
            }
        };
        let stream = Box::pin(stream);
        Ok(stream)
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

#[allow(unused)]
#[cfg(test)]
mod test {
    use futures::StreamExt;

    use super::*;
    use crate::test::*;

    #[tokio::test]
    async fn test_topic() -> anyhow::Result<()> {
        let wave = WaveClient::new().await?;
        let client = Client::mock(wave.node()).await?;
        let msg = "hello".to_string();
        let author = wave.make_author().await?;
        let mut stream = client.subscribe().await?;
        client.publish(&author, &msg).await?;
        let res: Message<String> = stream.next().await.unwrap()?;

        assert_eq!(msg, res.into_data());

        Ok(())
    }
}
