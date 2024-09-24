use error::Result;
pub use session::Session;
use session::{Meta, SessionId};
use wave_core::{author::Author, KVStore};

pub mod error;
pub mod message;
pub mod session;

pub struct Client<'store, T> {
    store: &'store T,
    author: Author,
}

impl<'store, T> Client<'store, T>
where
    T: wave_core::MakeStore,
{
    pub async fn create_session<'author>(
        &'author self,
        name: &str,
    ) -> Result<Session<T::Store<'author>>> {
        let (id, doc) = self.store.make(&self.author).await?;
        let meta = Meta::new(name.to_string())?;
        doc.insert("name", name).await?;
        let session = Session::new(id.as_ref().into(), meta, doc);
        Ok(session)
    }

    pub async fn get_session<'author: 'store>(
        &'author self,
        id: &SessionId,
    ) -> Result<Option<Session<T::Store<'author>>>> {
        let doc = self.store.get_store(&self.author, id).await?;

        if let Some(doc) = doc {
            let name = doc.get("name").await?;
            return name
                .map(|name| -> Result<Session<T::Store<'store>>> {
                    let meta = Meta::new(name)?;
                    Ok(Session::new(*id, meta, doc))
                })
                .transpose();
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
        let res = client.get_session(session.id()).await?.unwrap();
        assert_eq!(session.meta(), res.meta());
        assert_eq!(session.id(), res.id());
        Ok(())
    }
}
