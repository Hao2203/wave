pub use crate::error::{Error, ErrorKind, Result};
use std::{borrow::Cow, net::SocketAddr, pin::Pin, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite};

pub mod error;
pub mod socks5;
#[cfg(test)]
mod tests;
pub mod util;

pub trait Connection: AsyncRead + AsyncWrite + Send {}

impl<T: AsyncRead + AsyncWrite + Send> Connection for T {}

pub struct Incoming<'a> {
    pub incoming: &'a mut (dyn Connection + Unpin + 'a),
    pub local_addr: SocketAddr,
}

#[async_trait::async_trait]
pub trait Proxy {
    async fn serve<'a>(
        &self,
        conn: &'a mut (dyn Connection + Unpin + 'a),
        local_addr: SocketAddr,
    ) -> Result<(ProxyInfo, Pin<Box<dyn Connection + 'a>>)>;

    fn first_packet_size(&self) -> usize;
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

pub struct MixedProxy {
    proxies: Vec<Arc<dyn Proxy + Send + Sync>>,
}

#[async_trait::async_trait]
impl Proxy for MixedProxy {
    async fn serve<'a>(
        &self,
        conn: &'a mut (dyn Connection + Unpin + 'a),
        local_addr: SocketAddr,
    ) -> Result<(ProxyInfo, Pin<Box<dyn Connection + 'a>>)> {
        todo!()
    }

    fn first_packet_size(&self) -> usize {
        todo!()
    }
}
