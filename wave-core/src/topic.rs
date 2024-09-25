use super::*;
use async_stream::stream;
use bytes::Bytes;
use futures::stream::BoxStream;
use iroh::{
    client::docs::{self, LiveEvent},
    docs::store::Query,
};
use resource::Record;
use std::future::Future;

pub trait Topic {
    fn get(
        &self,
        key: impl AsRef<[u8]> + Send,
    ) -> impl Future<Output = Result<Option<Record>>> + Send;

    fn publish(
        &self,
        author: &Author,
        key: impl Into<Bytes> + Send,
        value: impl Into<Bytes> + Send,
    ) -> impl Future<Output = Result<()>> + Send;

    fn subscribe(&self) -> impl Future<Output = Result<BoxStream<Result<Record>>>> + Send;
}

impl Topic for docs::Doc {
    async fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Record>> {
        let entry = self.get_one(Query::key_exact(key)).await?;

        if let Some(entry) = entry {
            let record = Record::from_entry(self, entry).await?;
            return Ok(Some(record));
        }

        Ok(None)
    }

    async fn publish(
        &self,
        author: &Author,
        key: impl Into<Bytes> + Send,
        value: impl Into<Bytes> + Send,
    ) -> Result<()> {
        let _res = self.set_bytes(author.id(), key, value).await?;
        Ok(())
    }

    async fn subscribe(&self) -> Result<BoxStream<Result<Record>>> {
        let stream = self.subscribe().await?;
        let stream = stream! {
            for await event in stream {
                match event? {
                    LiveEvent::InsertLocal { entry } | LiveEvent::InsertRemote { entry, .. } => {
                        let record = Record::from_entry(self, entry).await?;
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

#[allow(unused)]
#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;
    use futures::StreamExt;

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
