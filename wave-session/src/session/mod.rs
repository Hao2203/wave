use crate::error::{ErrorKind, Result};
use derive_more::AsRef;
use iroh::docs::NamespaceId;
use serde::{Deserialize, Serialize};

pub mod actor;
pub mod index;
pub mod stats;

#[derive(Debug, Clone, PartialEq)]
pub struct Session<T> {
    id: SessionId,
    meta: Meta,
    store: T,
}

impl<T> Session<T> {
    pub fn new(id: SessionId, meta: Meta, store: T) -> Self {
        Self { id, meta, store }
    }

    pub fn id(&self) -> &SessionId {
        &self.id
    }

    pub fn meta(&self) -> &Meta {
        &self.meta
    }
}

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
pub struct Meta {
    pub name: String,
}

impl Meta {
    pub fn new(name: String) -> Result<Self> {
        if name.len() > 64 {
            return Err(ErrorKind::SessionNameTooLong.into());
        }
        Ok(Self { name })
    }
}

pub trait SessionMetadata {
    fn name(&self) -> &str;
}
