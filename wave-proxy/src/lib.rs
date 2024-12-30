pub use crate::error::{Error, ErrorKind, Result};
use error::Context;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
};

pub mod error;
pub mod socks5;
#[cfg(test)]
mod tests;
pub mod util;

#[async_trait::async_trait]
pub trait Proxy: Send + Sync {
    async fn proxy(&self, conn: Box<dyn Io>) -> Result<(Info, Box<dyn Io>)>;
}

pub struct Info {
    pub target: Target,
}

pub trait Io: AsyncRead + AsyncWrite + Send + Unpin {}

impl<T: AsyncRead + AsyncWrite + Send + Unpin> Io for T {}

pub struct ProxyServer {
    listener: TcpListener,
    proxy: Arc<dyn Proxy>,
}

impl ProxyServer {
    pub fn builder<T: Proxy>(proxy: T) -> Builder<T> {
        Builder::new(proxy)
    }

    pub async fn accept(&self) -> Result<Incoming> {
        let (io, source_addr) = self.listener.accept().await.context("accept failed")?;
        let proxy = self.proxy.clone();
        Ok(Incoming {
            proxy,
            io: Box::new(io),
            source_addr,
        })
    }
}

pub struct Builder<T> {
    addrs: Vec<SocketAddr>,
    proxy: T,
}

impl<T> Builder<T> {
    pub fn new(proxy: T) -> Self {
        Self {
            addrs: Vec::new(),
            proxy,
        }
    }

    pub fn bind(mut self, addr: SocketAddr) -> Self {
        self.addrs.push(addr);
        self
    }
}

impl<T> Builder<T>
where
    T: Proxy + 'static,
{
    pub async fn build(self) -> Result<ProxyServer> {
        let addrs = self.addrs.as_slice();
        let listener = TcpListener::bind(addrs).await.context("bind failed")?;
        let proxy = Arc::new(self.proxy);
        Ok(ProxyServer { listener, proxy })
    }
}

pub struct ProxyConnection {
    pub target: Target,
    pub io: Box<dyn Io>,
    pub source_addr: SocketAddr,
}

pub struct Incoming {
    proxy: Arc<dyn Proxy>,
    io: Box<dyn Io>,
    source_addr: SocketAddr,
}

impl Incoming {
    pub async fn start_proxy(self) -> Result<ProxyConnection> {
        let (info, io) = self.proxy.proxy(self.io).await?;
        Ok(ProxyConnection {
            target: info.target,
            io,
            source_addr: self.source_addr,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Target {
    Ip(SocketAddr),
    Domain(String, u16),
}
