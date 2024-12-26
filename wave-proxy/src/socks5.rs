use std::net::SocketAddr;

use fast_socks5::{server::Socks5Server, util::target_addr::TargetAddr, SocksError};
use futures_lite::{
    stream::{self, Boxed},
    StreamExt,
};

use crate::{error::ErrorKind, Incoming, ProxyBuilder, Result};

pub struct Socks5 {
    addr: SocketAddr,
}

impl Socks5 {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    async fn socks5_stream(self) -> Result<Boxed<Result<Incoming>>> {
        let server: Socks5Server = Socks5Server::bind(self.addr).await?;
        let stream = stream::unfold(server, |server| async move {
            let mut incoming = server.incoming();
            let res = if let Some(res) = incoming.next().await {
                Some(match res {
                    Err(e) => {
                        tracing::warn!("failed to accept incoming: {}", e);
                        Err(e.into())
                    }
                    Ok(socks5) => {
                        let socks5 = socks5.upgrade_to_socks5().await.unwrap();
                        let target_addr = socks5.target_addr().unwrap().clone();
                        let io = socks5.into_inner();
                        Ok(Incoming::new(target_addr, io))
                    }
                })
            } else {
                None
            };
            drop(incoming);
            res.map(|res| (res, server))
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

impl From<SocksError> for crate::Error {
    fn from(e: SocksError) -> Self {
        Self::with_source(ErrorKind::Unexpected, e)
    }
}
