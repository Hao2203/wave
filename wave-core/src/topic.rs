use super::*;
use async_stream::stream;
use author::Author;
use futures::stream::BoxStream;
use iroh::{
    client::docs::{self, LiveEvent},
    docs::store::Query,
};
use serde::{de::DeserializeOwned, Serialize};

pub trait Topic<K, V> {
    fn get(
        &self,
        key: &K,
    ) -> impl std::future::Future<Output = Result<Option<Record<K, V>>>> + Send;

    fn subscribe(
        &self,
    ) -> impl std::future::Future<Output = Result<BoxStream<Result<Record<K, V>>>>> + Send;
}

pub struct Client {
    doc: docs::Doc,
}

impl Client {
    #[cfg(test)]
    pub async fn mock(node: &iroh::client::Iroh) -> Result<Self> {
        let doc = node.docs().create().await?;
        Ok(Self { doc })
    }
}

impl<K, V> Topic<K, V> for Client
where
    for<'a> K: Serialize + DeserializeOwned + Send + Sync + 'a,
    for<'a> V: Serialize + DeserializeOwned + Send + Sync + 'a,
{
    async fn get(&self, key: &K) -> Result<Option<Record<K, V>>> {
        let entry = self
            .doc
            .get_one(Query::key_exact(rmp_serde::to_vec(key)?))
            .await?;

        if let Some(entry) = entry {
            let record = Record::from_entry(&self.doc, entry).await?;
            return Ok(Some(record));
        }

        Ok(None)
    }

    async fn subscribe(&self) -> Result<BoxStream<Result<Record<K, V>>>> {
        let stream = self.doc.subscribe().await?;
        let stream = stream! {
            for await event in stream {
                match event? {
                    LiveEvent::InsertLocal { entry } | LiveEvent::InsertRemote { entry, .. } => {
                        let record = Record::from_entry(&self.doc, entry).await?;
                        yield Ok(record)
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
pub struct Record<K, T> {
    pub id: K,
    pub data: T,
    pub author: Author,
    pub timestamp: u64,
}

impl<K, V> Record<K, V> {
    pub fn new(id: K, data: V, author: Author, timestamp: u64) -> Self {
        Self {
            id,
            data,
            author,
            timestamp,
        }
    }
}

impl<K, V> Record<K, V>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    pub async fn from_entry(docs: &docs::Doc, entry: docs::Entry) -> Result<Self> {
        let author = Author::new(entry.author());
        let data = entry.content_bytes(docs).await?;
        let data = rmp_serde::from_slice(&data)?;
        let timestamp = entry.timestamp();
        Ok(Record::new(
            rmp_serde::from_slice(entry.key())?,
            data,
            author,
            timestamp,
        ))
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
        // let wave = WaveClient::mock().await?;
        // let client = Client::mock(wave.node()).await?;
        // let msg = "hello".to_string();
        // let author = wave.make_author().await?;
        // let mut stream = client.subscribe().await?;
        // client.publish(&author, &msg).await?;
        // let res: Message<String> = stream.next().await.unwrap()?;

        // assert_eq!(msg, res.into_data());

        Ok(())
    }
}
