#![allow(unused)]

use super::*;
use crate::{
    author::{Author, CurrentAuthor},
    error::{ErrorKind, Result},
    message::{content::Content, Message},
};
use async_channel::{Receiver, Sender};
use async_stream::{stream, try_stream};
use chrono::Utc;
use futures::{stream::BoxStream, TryFutureExt};
use iroh::docs::store::Query;

pub struct Actor {
    author: Author,
    docs: iroh::client::docs::Doc,
    stream: Option<BoxStream<'static, Result<Message>>>,
}

impl Actor {
    pub async fn new(author: &impl CurrentAuthor, docs: iroh::client::docs::Doc) -> Result<Self> {
        let author_id = author.id();
        let name = docs
            .get_one(Query::key_exact("name").author(author_id.into()))
            .await?
            .ok_or(ErrorKind::AuthorNotFound)?
            .content_bytes(&docs)
            .await?;
        let author = Author::from_bytes(&name, author_id.into());

        let docs_clone = docs.clone();
        let stream = stream! {
            let mut stream = docs_clone.subscribe().await?;
            for await event in stream {
                let author = Author::new("".to_string(), [0; 32].into()).unwrap();
                let message = Message::new(author, Content::new("hello".to_string()).unwrap(), Utc::now().timestamp() as u64);
                yield Ok(message);
            }
        };
        let stream = Box::pin(stream) as BoxStream<'static, Result<Message>>;

        Ok(Self {
            author,
            docs,
            stream: None,
        })
    }
}
