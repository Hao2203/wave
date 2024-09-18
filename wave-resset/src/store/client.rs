#![allow(unused)]

use super::*;
use futures::{stream::StreamExt, TryStreamExt};
use iroh::{
    client::{
        blobs::{self, BlobStatus},
        docs,
    },
    docs::{store::Query, AuthorId, NamespaceId},
};
use serde::{Deserialize, Serialize};
use std::slice::SplitMut;

pub struct Client<'a> {
    blobs_client: &'a blobs::Client,
    author: AuthorId,
}

impl FileContentStore for Client<'_> {
    type Hash = [u8; 32];

    async fn get_size(&self, hash: &Self::Hash) -> Result<Option<usize>> {
        let res = self.blobs_client.status(hash.into()).await?;
        match res {
            BlobStatus::Complete { size, .. } => Ok(Some(size as usize)),
            BlobStatus::Partial { size: _ } => Ok(None),
            BlobStatus::NotFound => Ok(None),
        }
    }

    async fn insert(&self, file_path: &Path) -> Result<Self::Hash> {
        let res = self
            .blobs_client
            .add_from_path(
                file_path.to_path_buf(),
                true,
                iroh::blobs::util::SetTagOption::Auto,
                blobs::WrapOption::NoWrap,
            )
            .await?
            .finish()
            .await?;
        Ok(res.hash.into())
    }
}
