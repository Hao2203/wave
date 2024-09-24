use super::*;
use anyhow::Result;
use iroh::docs::AuthorId;
use serde::{Deserialize, Serialize};
use std::future::Future;

pub trait AuthorStore {
    fn default_author(&self) -> impl Future<Output = Result<Author>> + Send;

    fn make_author(&self) -> impl Future<Output = Result<Author>> + Send;
}

impl<T: iroh::blobs::store::Store> AuthorStore for iroh::node::Node<T> {
    async fn default_author(&self) -> Result<Author> {
        let id = self.authors().default().await?;
        Ok(Author { id })
    }
    async fn make_author(&self) -> Result<Author> {
        let id = self.authors().create().await?;
        Ok(Author { id })
    }
}

pub trait CurrentAuthor {
    fn current_author(&self) -> &Author;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Author {
    id: AuthorId,
}

impl Author {
    pub fn new(id: AuthorId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &AuthorId {
        &self.id
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        self.id.as_bytes()
    }
}

impl CurrentAuthor for Author {
    fn current_author(&self) -> &Author {
        self
    }
}

impl CurrentAuthor for DocStore<'_> {
    fn current_author(&self) -> &Author {
        self.author
    }
}
