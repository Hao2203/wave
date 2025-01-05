pub use crate::error::{Error, ErrorKind, Result};
use std::{borrow::Cow, net::SocketAddr, pin::Pin, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite};
use util::Cloneable;

pub mod error;
pub mod socks5;
#[cfg(test)]
mod tests;
pub mod util;

pub trait Connection: AsyncRead + AsyncWrite + Send {}

impl<T: AsyncRead + AsyncWrite + Send> Connection for T {}

pub type BoxConn<'a> = Pin<Box<dyn Connection + 'a>>;

#[async_trait::async_trait]
pub trait Proxy {
    async fn serve<'a>(
        &self,
        conn: BoxConn<'a>,
        local_addr: SocketAddr,
    ) -> Result<(ProxyInfo, BoxConn<'a>)>;

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

#[derive(Default)]
pub struct MixedProxy {
    proxies: Vec<Arc<dyn Proxy + Send + Sync>>,
    first_packet_size: usize,
}

impl MixedProxy {
    pub fn new() -> Self {
        Self {
            proxies: Vec::new(),
            first_packet_size: 0,
        }
    }

    pub fn add_proxy(&mut self, proxy: Arc<dyn Proxy + Send + Sync>) {
        self.first_packet_size = self.first_packet_size.max(proxy.first_packet_size());
        self.proxies.push(proxy);
    }

    pub fn first_packet_size(&self) -> usize {
        self.first_packet_size
    }
}

#[async_trait::async_trait]
impl Proxy for MixedProxy {
    async fn serve<'a>(
        &self,
        conn: BoxConn<'a>,
        local_addr: SocketAddr,
    ) -> Result<(ProxyInfo, BoxConn<'a>)> {
        let conn = util::BufConnManager::new(conn, self.first_packet_size);
        let conn = Cloneable::new(conn);

        for proxy in self.proxies.iter() {
            conn.value.lock().unwrap().reset();

            match proxy.serve(Box::pin(conn.clone()), local_addr).await {
                Err(e) if e.kind() == ErrorKind::UnSupportedProxyProtocol => {
                    continue;
                }
                res => {
                    return res;
                }
            }
        }

        Err(Error::new(
            ErrorKind::UnSupportedProxyProtocol,
            "Unsupported proxy protocol in mixed proxy",
        ))
    }

    fn first_packet_size(&self) -> usize {
        self.first_packet_size
    }
}
