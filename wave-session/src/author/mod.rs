use crate::error::{ErrorKind, Result};
use iroh::docs::AuthorId;
use serde::{Deserialize, Serialize};

pub trait CurrentAuthor {
    fn id(&self) -> [u8; 32];
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Author {
    name: String,
    id: AuthorId,
}

impl Author {
    pub fn new(name: String, id: AuthorId) -> Result<Self> {
        if name.len() > 64 {
            return Err(ErrorKind::AuthorNameTooLong)?;
        }
        Ok(Self { name, id })
    }

    pub fn from_bytes(name: &[u8], id: AuthorId) -> Self {
        Self {
            name: String::from_utf8_lossy(name).to_string(),
            id,
        }
    }
}

impl CurrentAuthor for Author {
    fn id(&self) -> [u8; 32] {
        self.id.into()
    }
}

impl CurrentAuthor for AuthorId {
    fn id(&self) -> [u8; 32] {
        (*self).into()
    }
}

impl CurrentAuthor for iroh::docs::Author {
    fn id(&self) -> [u8; 32] {
        self.id().to_bytes()
    }
}

impl CurrentAuthor for iroh::docs::AuthorPublicKey {
    fn id(&self) -> [u8; 32] {
        *self.as_bytes()
    }
}

impl CurrentAuthor for [u8; 32] {
    fn id(&self) -> [u8; 32] {
        *self
    }
}
