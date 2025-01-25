pub use crate::{
    error::{Error, ErrorKind, Result},
    // server::{Builder, ProxyServer},
};

use derive_more::derive::{Display, From};
use std::{
    future::Future,
    io::Cursor,
    net::SocketAddr,
    pin::{pin, Pin},
    str::FromStr,
    sync::Arc,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

pub mod error;
pub mod server;
pub mod services;
#[cfg(test)]
mod tests;

pub trait Connection: AsyncRead + AsyncWrite + Send {}

impl<T: AsyncRead + AsyncWrite + Send> Connection for T {}

pub trait ProxyApp {
    type Ctx: Send + Sync;
    type Upstream: Connection + Sync + Unpin;
    fn new_ctx(&self) -> Self::Ctx;

    fn upstream(
        &self,
        ctx: &mut Self::Ctx,
        target: &Address,
    ) -> impl Future<Output = Result<Option<Self::Upstream>>> + Send;

    fn after_forward(
        &self,
        _ctx: &mut Self::Ctx,
        _tunnel: Self::Upstream,
    ) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }
}

impl<T> ProxyApp for Arc<T>
where
    T: ProxyApp,
{
    type Ctx = T::Ctx;
    type Upstream = T::Upstream;

    fn new_ctx(&self) -> Self::Ctx {
        self.as_ref().new_ctx()
    }

    fn upstream(
        &self,
        ctx: &mut Self::Ctx,
        target: &Address,
    ) -> impl Future<Output = Result<Option<Self::Upstream>>> + Send {
        self.as_ref().upstream(ctx, target)
    }

    fn after_forward(
        &self,
        ctx: &mut Self::Ctx,
        tunnel: Self::Upstream,
    ) -> impl Future<Output = Result<()>> + Send {
        self.as_ref().after_forward(ctx, tunnel)
    }
}

#[async_trait::async_trait]
pub trait ProxyService<A: ProxyApp> {
    async fn serve<'a>(&self, app: &A, incoming: Incoming<'a>) -> Result<ProxyStatus<'a>>;
}

pub type BoxConn<'a> = Pin<Box<dyn Connection + 'a>>;

pub struct Incoming<'a> {
    io_buf: Cursor<bytes::Bytes>,
    pub conn: &'a mut (dyn Connection + Unpin + 'a),
    pub local_addr: SocketAddr,
}

impl AsyncRead for Incoming<'_> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let Incoming {
            io_buf,
            conn,
            local_addr: _,
        } = self.get_mut();
        let reader = pin!(io_buf.chain(conn));
        reader.poll_read(cx, buf)
    }
}

impl AsyncWrite for Incoming<'_> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let writer = Pin::new(&mut self.get_mut().conn);
        writer.poll_write(cx, buf)
    }
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let writer = Pin::new(&mut self.get_mut().conn);
        writer.poll_flush(cx)
    }
    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let writer = Pin::new(&mut self.get_mut().conn);
        writer.poll_shutdown(cx)
    }
}

pub enum ProxyStatus<'a> {
    Success,
    Continue(Incoming<'a>),
}

#[derive(Debug, Display, From, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Address {
    #[from]
    Ip(SocketAddr),
    #[display("{_0}:{_1}")]
    Domain(Arc<str>, u16),
}

impl FromStr for Address {
    type Err = AddressFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(ip) = s.parse() {
            return Ok(Address::Ip(ip));
        }
        if let Some(colon) = s.find(':') {
            let domain = s[0..colon].to_string();
            let port = s[colon + 1..].parse::<u16>()?;
            return Ok(Address::Domain(Arc::from(domain), port));
        }
        Err(AddressFromStrErr::Other(s.to_string()))
    }
}

#[derive(Debug, From, Display, derive_more::Error)]
#[error(ignore)]
pub enum AddressFromStrErr {
    #[from]
    IpParseError(std::net::AddrParseError),
    #[from]
    IntParseError(std::num::ParseIntError),
    #[display("Parse address failed: {_0}")]
    Other(String),
}
