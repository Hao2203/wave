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
        let (id, doc) = self.store.make(&self.author).await?;
        let session = Session::new(id.as_ref().into(), name.to_string())?;
        doc.insert("name", &name).await?;
        Ok(session)
    }

    pub async fn get_session(&self, id: &SessionId) -> Result<Option<Session>> {
        let doc = self.store.get_store(&self.author, id).await?;

        if let Some(doc) = doc {
            let name = doc.get("name").await?;
            return name.map(|name| Session::new(*id, name)).transpose();
        };

        Ok(None)
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use super::*;
    use wave_core::{author::AuthorStore, WaveClient};

    #[tokio::test]
    async fn test() -> Result<()> {
        let client = iroh::node::MemNode::memory().spawn().await?;
        let author = client.make_author().await?;
        let client = Client {
            store: &client,
            author,
        };
        let session = client.create_session("test").await?;
        let res = client.get_session(session.id()).await?;
        assert_eq!(session, res.unwrap());
        Ok(())
    }
}
