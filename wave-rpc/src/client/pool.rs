use std::future::Future;

use deadpool::managed::Manager;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait MakeConnection {
    fn make_connection(&self) -> impl Future<Output = impl DynConnection> + Send;
}

pub trait DynConnection: AsyncRead + AsyncWrite + Send + Unpin {}

impl<T> DynConnection for T where T: AsyncRead + AsyncWrite + Send + Unpin {}

struct Connection<T> {
    io: T,
}

pub struct Pool<'a, T> {
    manger: &'a T,
}

impl<'a, T> Manager for Pool<'a, T>
where
    T: MakeConnection + Send + Sync,
{
    type Type = Box<dyn DynConnection + Send + 'a>;
    type Error = std::io::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let conn = self.manger.make_connection().await;
        Ok(Box::new(conn))
    }

    async fn recycle(
        &self,
        obj: &mut Self::Type,
        metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        Ok(())
    }
}
