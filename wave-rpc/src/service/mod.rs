use anyhow::Result;
use futures::future::BoxFuture;
use std::future::Future;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait Service<Req> {
    type Response;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response>> + Send;
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

pub trait Handle<'a, Conn: ?Sized> {
    fn handle<'conn>(&'a self, conn: &'conn mut Conn) -> BoxFuture<'conn, anyhow::Result<()>>
    where
        'a: 'conn;
}
