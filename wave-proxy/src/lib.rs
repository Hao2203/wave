pub use crate::error::{Error, ErrorKind, Result};
use std::{borrow::Cow, net::SocketAddr};
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

pub trait ProxyBuilder {
    fn build(
        &self,
        incoming: Incoming<'_>,
    ) -> impl std::future::Future<Output = Result<impl Proxy>> + Send;
}

#[async_trait::async_trait]
pub trait Proxy: Send {
    fn proxy_info(&self) -> &ProxyInfo;

    async fn tunnel(&mut self) -> Result<&mut (dyn Connection + Unpin)>;
}

#[async_trait::async_trait]
impl Proxy for Box<dyn Proxy> {
    fn proxy_info(&self) -> &ProxyInfo {
        self.as_ref().proxy_info()
    }

    async fn tunnel(&mut self) -> Result<&mut (dyn Connection + Unpin)> {
        self.as_mut().tunnel().await
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

pub struct Builder<T> {
    proxy: T,
}

impl<T> Builder<T> {
    pub fn new(proxy: T) -> Self {
        Self { proxy }
    }
}

impl<T> Builder<T>
where
    T: ProxyBuilder,
{
    pub async fn serve<'a>(
        &'a self,
        io: &'a mut (impl Connection + Unpin),
        local_addr: SocketAddr,
    ) -> Result<ProxyHandler<'a>> {
        let incoming = Incoming {
            incoming: io,
            local_addr,
        };
        let proxy = self
            .proxy
            .build(incoming)
            .await
            .map(|proxy| Box::new(proxy) as Box<dyn Proxy>)?;
        Ok(ProxyHandler { proxy })
    }
}

pub struct ProxyHandler<'a> {
    proxy: Box<dyn Proxy + 'a>,
}

impl<'a> ProxyHandler<'a> {
    pub async fn tunnel(&mut self) -> Result<&mut (dyn Connection + Unpin)> {
        self.proxy.as_mut().tunnel().await
    }

    pub fn info(&self) -> &ProxyInfo {
        self.proxy.as_ref().proxy_info()
    }

    pub async fn direct(&mut self) -> Result<()> {
        todo!()
    }
}
