use derive_more::AsRef;
use wave_core::NodeList;

pub mod record;

#[derive(Debug, AsRef)]
#[as_ref(forward)]
pub struct SessionId([u8; 32]);

#[derive(Debug)]
pub struct Session {
    id: SessionId,
    nodes: NodeList,
}

impl Session {
    pub fn create(id: SessionId) -> Self {
        Self {
            id,
            nodes: NodeList::new(),
        }
    }

    pub fn id(&self) -> &SessionId {
        &self.id
    }

    pub fn nodes(&self) -> &NodeList {
        &self.nodes
    }
}
