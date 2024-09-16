use derive_more::AsRef;
use serde::{Deserialize, Serialize};
use wave_core::{node::NodeList, Entity};

#[derive(Debug, AsRef)]
#[as_ref([u8], [u8; 32])]
pub struct SessionId([u8; 32]);

#[derive(Debug)]
pub struct Session {
    id: SessionId,
    data: SessionData,
}

impl Session {
    pub fn create(id: SessionId, data: SessionData) -> Self {
        Self { id, data }
    }

    pub fn id(&self) -> &SessionId {
        &self.id
    }

    pub fn data(&self) -> &SessionData {
        &self.data
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {}

impl Entity for Session {
    type Id = SessionId;
    type Data = SessionData;
}
