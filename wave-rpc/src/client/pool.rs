use deadpool::managed::Manager;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait MakeConnection {
    fn make_connection(&self) -> (impl AsyncRead + Unpin, impl AsyncWrite + Unpin);
}

struct Connection {
    reader: Box<dyn AsyncRead + Send + Unpin>,
    writer: Box<dyn AsyncWrite + Send + Unpin>,
}

impl Connection {}

pub struct Pool<T> {
    manger: T,
}

impl<T> Manager for Pool<T>
where
    T: MakeConnection + Send + Sync,
{
    type Type = Connection;
    type Error = std::io::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let (reader, writer) = self.manger.make_connection();
        Ok(Connection { reader, writer })
    }

    async fn recycle(
        &self,
        obj: &mut Self::Type,
        metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        Ok(())
    }
}
