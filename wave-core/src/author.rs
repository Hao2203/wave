use iroh::docs::AuthorId;
use serde::{Deserialize, Serialize};

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
