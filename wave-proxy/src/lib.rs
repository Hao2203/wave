pub use crate::error::{Error, ErrorKind, Result};
use std::{
    borrow::Cow,
    net::SocketAddr,
    pin::{pin, Pin},
};
use tokio::io::{AsyncRead, AsyncWrite};
use util::IoPreHandler;

pub mod error;
// pub mod socks5;
// #[cfg(test)]
// mod tests;
pub mod util;

pub trait Connection: AsyncRead + AsyncWrite + Send {}

impl<T: AsyncRead + AsyncWrite + Send> Connection for T {}

pub type BoxConnection = Pin<Box<dyn Connection>>;

#[async_trait::async_trait]
pub trait ProxyCtx: Send {
    async fn upstream_session(&mut self, info: &ProxyInfo) -> Result<Option<UpstreamSession>>;
}

#[async_trait::async_trait]
pub trait Proxy: Sync {
    const ROUTE_SIZE: usize;

    async fn proxy_incoming(
        &self,
        ctx: &mut dyn ProxyCtx,
        incoming: &mut (dyn Connection + Unpin),
    ) -> Result<()>;
}

pub struct ProxyChain<T1, T2>(T1, T2);

#[async_trait::async_trait]
impl<T1, T2> Proxy for ProxyChain<T1, T2>
where
    T1: Proxy,
    T2: Proxy,
{
    const ROUTE_SIZE: usize = if T1::ROUTE_SIZE > T2::ROUTE_SIZE {
        T1::ROUTE_SIZE
    } else {
        T2::ROUTE_SIZE
    };

    async fn proxy_incoming(
        &self,
        ctx: &mut dyn ProxyCtx,
        incoming: &mut (dyn Connection + Unpin),
    ) -> Result<()> {
        let res = self.0.proxy_incoming(ctx, incoming).await;
        if let Err(e) = &res {
            if e.kind() == ErrorKind::UnSupportedProxyProtocol {
                return self.1.proxy_incoming(ctx, incoming).await;
            }
        }
        res
    }
}

pub struct ClientSession {
    pub downstream: BoxConnection,
    pub source_addr: SocketAddr,
}

pub struct UpstreamSession {
    pub upstream: BoxConnection,
}

pub struct ProxyInfo {
    pub proxy_mode: Cow<'static, str>,
    pub target: Target,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Target {
    Ip(SocketAddr),
    Domain(String, u16),
}

pub struct Builder<T> {
    proxy: T,
}

impl<T> Builder<T> {
    pub fn new(proxy: T) -> Self {
        Self { proxy }
    }

    pub fn add_proxy<T2>(self, proxy: T2) -> Builder<ProxyChain<T, T2>> {
        let proxy = ProxyChain(self.proxy, proxy);
        Builder { proxy }
    }
}

impl<T> Builder<T>
where
    T: Proxy,
{
    pub async fn serve(&self, mut ctx: impl ProxyCtx, io: impl Connection) -> Result<()> {
        let mut io = IoPreHandler::new(io, T::ROUTE_SIZE);
        let mut incoming = pin!(io);
        self.proxy.proxy_incoming(&mut ctx, &mut incoming).await
    }
}
