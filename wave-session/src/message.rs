use content::Content;
pub use error::*;
use serde::{Deserialize, Serialize};
use wave_core::author::Author;

pub mod content;
pub mod error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    author: Author,
    content: Content,
    timestamp: u64,
}

impl Message {
    pub fn new(author: Author, content: Content, timestamp: u64) -> Self {
        Self {
            author,
            content,
            timestamp,
        }
    }
}
