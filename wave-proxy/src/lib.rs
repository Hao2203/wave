pub use crate::error::{Error, ErrorInner, Result};
use bytes::{Bytes, BytesMut};
use std::{
    borrow::Cow,
    net::SocketAddr,
    pin::{pin, Pin},
    sync::Arc,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

pub mod error;
pub mod socks5;
// #[cfg(test)]
// mod tests;
pub mod util;

pub trait Connection: AsyncRead + AsyncWrite + Send {}

impl<T: AsyncRead + AsyncWrite + Send> Connection for T {}

pub type BoxConn<'a> = Pin<Box<dyn Connection + 'a>>;

pub struct Incoming<T> {
    io_buf: Bytes,
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

#[async_trait::async_trait]
pub trait Proxy<T> {
    async fn serve<'a>(&self, incoming: Incoming<T>) -> Result<ProxyStatus<'a, T>>
    where
        T: 'a;
}

pub enum ProxyStatus<'a, T> {
    Success(ProxyInfo<'a>),
    Continue(T),
}

pub struct ProxyInfo<'a> {
    pub proxy_mode: Cow<'static, str>,
    pub target: Target,
    pub tunnel: BoxConn<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Target {
    Ip(SocketAddr),
    Domain(String, u16),
}

#[derive(Default)]
pub struct Builder<T> {
    proxies: Vec<Arc<dyn Proxy<T> + Send + Sync>>,
}

impl<T> Builder<T> {
    pub fn new() -> Self {
        Self {
            proxies: Vec::new(),
        }
    }

    pub fn add_proxy(&mut self, proxy: impl Proxy<T> + Send + Sync + 'static) {
        self.proxies.push(Arc::new(proxy));
    }

    pub fn build(self) -> Result<MixedProxy<T>> {
        Ok(MixedProxy {
            proxies: self.proxies.into(),
        })
    }
}

#[derive(Default)]
pub struct MixedProxy<T> {
    proxies: Arc<[Arc<dyn Proxy<T> + Send + Sync>]>,
}

impl<T> MixedProxy<T>
where
    T: AsyncRead + AsyncWrite + Send + Unpin,
{
    pub async fn serve<'a>(&self, mut conn: T, local_addr: SocketAddr) -> Result<ProxyInfo> {
        let mut buf = BytesMut::with_capacity(1024);
        conn.read_buf(&mut buf).await.unwrap();
        let buf = buf.freeze();

        for proxy in self.proxies.iter() {
            let incoming = Incoming {
                io_buf: buf.clone(),
                conn,
                local_addr,
            };

            match proxy.serve(incoming).await? {
                ProxyStatus::Success(info) => return Ok(info),
                ProxyStatus::Continue(io) => conn = io,
            }
        }

        Err(Error::new(
            ErrorInner::UnSupportedProxyProtocol,
            "Unsupported proxy protocol in mixed proxy",
        ))
    }
}
