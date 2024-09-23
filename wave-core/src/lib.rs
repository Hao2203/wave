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

pub struct WaveClient {
    node: iroh::node::FsNode,
}

impl WaveClient {
    pub async fn mock() -> Result<Self> {
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
