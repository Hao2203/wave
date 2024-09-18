use std::borrow::Cow;

use super::*;
use bytes::{BufMut, Bytes, BytesMut};
use futures::{StreamExt, TryStreamExt};
use iroh::{
    client::docs,
    docs::{store::Query, AuthorId, Entry},
};
use wave_core::Key;

pub struct RecordsClient<'a> {
    doc: &'a docs::Doc,
    author: AuthorId,
}

impl RecordStore for RecordsClient<'_> {
    async fn list(&self) -> Result<impl Stream<Item = Result<Record>>> {
        let query = Query::author(self.author).build();
        let res = self
            .doc
            .get_many(query)
            .await?
            .map_err(anyhow::Error::from)
            .and_then(move |e| async move {
                let res = e.content_bytes(self.doc).await?;
                let record = serde_json::from_slice(&res)?;
                Ok(record)
            });
        Ok(res)
    }
}

impl RecordStoreMut for RecordsClient<'_> {
    async fn insert_file(&self, file: &FileRef) -> Result<()> {
        todo!()
    }

    async fn insert_text(&self, text: &Text) -> Result<()> {
        todo!()
    }
}
