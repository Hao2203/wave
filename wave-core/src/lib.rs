use anyhow::Result;

pub mod author;
pub mod node;
pub mod resource;
pub mod store;
pub mod topic;

pub use store::{KVStore, MakeStore};

#[cfg(test)]
pub mod test;

use author::Author;
use iroh::docs::NamespaceId;
use store::DocStore;

pub struct WaveClient {
    node: iroh::node::FsNode,
}

impl WaveClient {
    pub async fn new() -> Result<Self> {
        let node = iroh::node::Node::persistent("/tmp").await?.spawn().await?;
        Ok(Self { node })
    }

    pub fn node(&self) -> &iroh::node::FsNode {
        &self.node
    }

    pub async fn make_author(&self) -> Result<Author> {
        let id = self.node.authors().default().await?;
        Ok(Author::new(id))
    }
}

impl MakeStore for WaveClient {
    type Store = DocStore;
    type Id = NamespaceId;
    async fn make(&self, author: &Author) -> Result<(Self::Id, Self::Store)> {
        let doc = self.node().docs().create().await?;
        Ok((doc.id(), DocStore::new(doc, *author.id())))
    }

    async fn get_store(
        &self,
        author: &Author,
        id: impl AsRef<[u8; 32]>,
    ) -> Result<Option<Self::Store>> {
        let doc = self.node().docs().open(id.as_ref().into()).await?;
        Ok(doc.map(|doc| DocStore::new(doc, *author.id())))
    }
}
