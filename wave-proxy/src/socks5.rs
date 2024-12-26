use std::net::SocketAddr;

use fast_socks5::{server::Socks5Server, util::target_addr::TargetAddr, SocksError};
use futures_lite::{
    stream::{self, Boxed},
    StreamExt,
};

use crate::{
    error::{Context, ErrorKind},
    Error, Incoming, ProxyBuilder, Result,
};

pub struct Socks5 {
    addr: SocketAddr,
}

impl Socks5 {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    async fn socks5_stream(self) -> Result<Boxed<Result<Incoming>>> {
        let server: Socks5Server = Socks5Server::bind(self.addr)
            .await
            .map_err(|e| Error::new(e.kind(), "failed to bind socks5 server"))?;

        let stream = stream::try_unfold(server, |server| async move {
            let mut incoming = server.incoming();

            if let Some(res) = incoming.next().await {
                let socks5 = res
                    .context("failed to accept socks5 connection")?
                    .upgrade_to_socks5()
                    .await
                    .context("failed to upgrade to socks5")?;

                let target_addr = socks5
                    .target_addr()
                    .ok_or(Error::new(ErrorKind::GetTargetFailed, "get target failed"))?
                    .clone();
                let io = socks5.into_inner();

                drop(incoming);

                return Ok(Some((Incoming::new(target_addr, io), server)));
            }

            Ok(None)
        });

        Ok(stream.boxed())
    }
}

impl ProxyBuilder for Socks5 {
    type Stream = Boxed<Result<Incoming>>;

    fn build(self) -> impl std::future::Future<Output = Result<Self::Stream>> + Send {
        self.socks5_stream()
    }
}

impl From<TargetAddr> for crate::Target {
    fn from(addr: TargetAddr) -> Self {
        match addr {
            TargetAddr::Ip(ip) => Self::Ip(ip),
            TargetAddr::Domain(domain, port) => Self::Domain(domain, port),
        }
    }
}

impl From<&SocksError> for ErrorKind {
    fn from(value: &SocksError) -> Self {
        type E = SocksError;
        match value {
            E::Io(e) => e.into(),
            E::InvalidHeader {
                expected: _,
                found: _,
            }
            | E::UnsupportedSocksVersion(_) => ErrorKind::UnSupportedProxyProtocol,
            _ => ErrorKind::Unexpected,
        }
    }
}
