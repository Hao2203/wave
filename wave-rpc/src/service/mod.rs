use anyhow::Result;
use std::future::Future;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait Service<Req> {
    type Response;
    type Key;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response>> + Send;

    fn key(&self) -> Self::Key;
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
