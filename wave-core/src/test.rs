#![allow(unused)]
use super::*;
use author::Author;

pub struct WaveClient {
    node: iroh::node::MemNode,
}

impl WaveClient {
    pub async fn new() -> Result<Self> {
        let node = iroh::node::Node::memory().spawn().await?;
        Ok(Self { node })
    }

    pub fn node(&self) -> &iroh::node::MemNode {
        &self.node
    }

    pub async fn make_author(&self) -> Result<Author> {
        let id = self.node.authors().default().await?;
        Ok(Author::new(id))
    }
}
