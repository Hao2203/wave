use serde::{Deserialize, Serialize};

use super::{SessionHandle, SessionId};

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStats {
    pub id: [u8; 32],
    pub name: String,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub users_connected: u64,
}

impl SessionHandle for SessionStats {
    fn id(&self) -> SessionId {
        self.id.into()
    }
}
