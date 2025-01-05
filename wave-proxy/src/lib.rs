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
pub trait ProxyService {
    async fn serve<'a>(
        &self,
        conn: Incoming<'a>,
    ) -> Result<(ProxyInfo, Pin<Box<dyn Connection + 'a>>)>;
}

#[async_trait::async_trait]
impl<T> ProxyService for Arc<T>
where
    T: ProxyService + ?Sized + Sync + Send,
{
    async fn serve<'a>(
        &self,
        conn: Incoming<'a>,
    ) -> Result<(ProxyInfo, Pin<Box<dyn Connection + 'a>>)> {
        let this = self.clone();
        this.serve(conn).await
    }
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
