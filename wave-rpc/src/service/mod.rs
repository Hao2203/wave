use anyhow::Result;
use futures::future::BoxFuture;
use std::future::Future;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait Service<'a> {
    type Request: 'a;
    type Response: 'a;

    fn call(
        &'a self,
        req: Self::Request,
    ) -> impl Future<Output = Result<Self::Response>> + Send + 'a;
}

/// ```rust
/// use wave_rpc::service::Connection;
/// use tokio::runtime::Runtime;
/// use tokio::net::TcpStream;
/// struct IsConnection<T: Connection>(T);
///
/// type Conn = IsConnection<TcpStream>;
///
/// ```
pub trait Connection: AsyncRead + AsyncWrite {}

impl<T: AsyncRead + AsyncWrite> Connection for T {}

pub trait Handle<Conn: ?Sized> {
    fn handle<'conn>(&self, conn: &'conn mut Conn) -> BoxFuture<'conn, anyhow::Result<()>>
    where
        Self: 'conn;
}
