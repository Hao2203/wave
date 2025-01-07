pub use crate::{
    error::{Error, ErrorInner, Result},
    server::{Builder, ProxyServer},
};

use std::{
    borrow::Cow,
    io::Cursor,
    net::SocketAddr,
    pin::{pin, Pin},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

pub mod error;
pub mod server;
pub mod socks5;
#[cfg(test)]
mod tests;

pub trait Connection: AsyncRead + AsyncWrite + Send {}

impl<T: AsyncRead + AsyncWrite + Send> Connection for T {}

#[async_trait::async_trait]
pub trait Proxy<T> {
    async fn serve<'a>(&self, incoming: Incoming<T>) -> Result<ProxyStatus<'a, T>>
    where
        T: 'a;
}

pub type BoxConn<'a> = Pin<Box<dyn Connection + 'a>>;

pub struct Incoming<T> {
    io_buf: Cursor<bytes::Bytes>,
    pub conn: T,
    pub local_addr: SocketAddr,
}

impl<T> AsyncRead for Incoming<T>
where
    T: AsyncRead + Unpin,
{
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

impl<T> AsyncWrite for Incoming<T>
where
    T: AsyncWrite + Unpin,
{
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

pub enum ProxyStatus<'a, T> {
    Success(ProxyHandler<'a>),
    Continue(T),
}

pub struct ProxyHandler<'a> {
    pub proxy_mode: Cow<'static, str>,
    pub target: Target,
    pub tunnel: BoxConn<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Target {
    Ip(SocketAddr),
    Domain(String, u16),
}
