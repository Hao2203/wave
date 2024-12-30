pub use crate::error::{Error, ErrorKind, Result};
use std::{borrow::Cow, net::SocketAddr};
use tokio::io::{AsyncRead, AsyncWrite};

pub mod error;
// pub mod socks5;
// #[cfg(test)]
// mod tests;
pub mod server;
pub mod util;

pub trait Io: AsyncRead + AsyncWrite + Send + Unpin {}

impl<T: AsyncRead + AsyncWrite + Send + Unpin> Io for T {}

#[async_trait::async_trait]
pub trait Proxy: Send + Sync {
    fn name(&self) -> Cow<'static, str>;

    async fn take_proxy_info(&self, client_conn: &mut ClientConn<'_>) -> Result<ProxyInfo>;

    /// forward client_conn to target_conn if target_conn is available
    async fn forward(
        &self,
        client_conn: &mut ClientConn<'_>,
        target_conn: &mut TargetConn<'_>,
    ) -> Result<()>;

    /// The method is called when the target connection is not available
    async fn fail_to_connect_target(
        &self,
        client_conn: &mut ClientConn<'_>,
        reason: FailedReason,
    ) -> Result<()>;
}

pub struct ClientConn<'a> {
    pub io: &'a mut dyn Io,
    pub source_addr: &'a SocketAddr,
}

pub struct TargetConn<'a> {
    pub io: &'a mut dyn Io,
    pub target_addr: &'a SocketAddr,
}

pub enum FailedReason {
    ConnectionNotAllowed,
    NetworkUnreachable,
    HostUnreachable,
    ConnectionRefused,
    ConnectionTimeout,
    TtlExpired,
}

pub struct ProxyInfo {
    pub target: Target,
    pub source_addr: SocketAddr,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Target {
    Ip(SocketAddr),
    Domain(String, u16),
}
