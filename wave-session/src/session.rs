use derive_more::AsRef;
use iroh::docs::NamespaceId;
use serde::{Deserialize, Serialize};

use crate::error::{ErrorKind, Result};

pub mod actor;
pub mod client;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, AsRef, Serialize, Deserialize,
)]
#[as_ref([u8], [u8; 32])]
pub struct SessionId([u8; 32]);

impl SessionId {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<SessionId> for NamespaceId {
    fn from(id: SessionId) -> Self {
        id.0.into()
    }
}

impl From<NamespaceId> for SessionId {
    fn from(id: NamespaceId) -> Self {
        Self(id.into())
    }
}

impl From<[u8; 32]> for SessionId {
    fn from(id: [u8; 32]) -> Self {
        Self(id)
    }
}

impl From<&[u8; 32]> for SessionId {
    fn from(id: &[u8; 32]) -> Self {
        Self(*id)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    id: SessionId,
    name: String,
}

impl Session {
    pub fn new(id: SessionId, name: String) -> Result<Self> {
        if name.len() > 64 {
            return Err(ErrorKind::SessionNameTooLong.into());
        }

        Ok(Self { id, name })
    }

    pub fn id(&self) -> &SessionId {
        &self.id
    }
}
