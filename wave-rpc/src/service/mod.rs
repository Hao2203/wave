use anyhow::Result;
use futures::future::BoxFuture;
use std::future::Future;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait Service {
    type Request;
    type Response;

    const ID: u64;
}

pub trait Call<S: Service> {
    fn call(&self, req: S::Request) -> impl Future<Output = Result<S::Response>> + Send;
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
