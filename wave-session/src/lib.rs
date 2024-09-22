use error::Result;
pub use session::Session;
use session::SessionId;
use wave_core::{author::Author, KVStore};

pub mod error;
pub mod message;
pub mod session;

pub struct Client<'a, T> {
    store: &'a T,
    author: Author,
}

impl<'a, T> Client<'a, T>
where
    T: wave_core::MakeStore,
{
    pub async fn create_session(&self, name: &str) -> Result<Session> {
        let doc = self.store.make(&self.author).await?;
        let id = SessionId::from(doc.id().as_ref());
        let session = Session::new(id, name.to_string())?;
        doc.insert("name", &name).await?;
        Ok(session)
    }

    pub async fn get_session(&self, id: SessionId) -> Result<Option<Session>> {
        let doc = self.store.get_store(&self.author, id).await?;
        if let Some(doc) = doc {
            let name = doc.get("name").await?;
            name.map(|name| Session::new(id, name)).transpose()
        } else {
            Ok(None)
        }
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use super::*;
}
