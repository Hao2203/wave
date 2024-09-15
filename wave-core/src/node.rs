use derive_more::{AsMut, AsRef, From};
use iroh::net::key::PublicKey;

#[derive(Debug, From, AsRef)]
#[from(forward)]
pub struct NodeId {
    #[as_ref(forward)]
    key: PublicKey,
}

impl NodeId {
    pub fn new(key: PublicKey) -> Self {
        Self { key }
    }

    pub fn key(&self) -> &PublicKey {
        &self.key
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        self.key.as_bytes()
    }

    pub fn as_slice(&self) -> &[u8] {
        self.key.as_ref()
    }
}

#[derive(Debug, AsRef, AsMut)]
pub struct NodeList {
    #[as_ref(forward)]
    #[as_mut(forward)]
    list: Vec<NodeId>,
}

impl NodeList {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }

    pub fn count(&self) -> usize {
        self.list.len()
    }
}

impl Default for NodeList {
    fn default() -> Self {
        Self::new()
    }
}
