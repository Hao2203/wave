use crate::{error::Context, Error, ErrorKind, Incoming, ProxyBuilder, Result, Target};
use fast_socks5::{
    server::{AcceptAuthentication, Config, Socks5Server},
    util::target_addr::TargetAddr,
    SocksError,
};
use futures_lite::{
    stream::{self, Boxed},
    StreamExt,
};
use std::net::SocketAddr;

pub struct Socks5 {
    addr: SocketAddr,
}

impl Socks5 {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    async fn socks5_stream(self) -> Result<Boxed<Result<Incoming>>> {
        let config = Config::<AcceptAuthentication>::default();
        let server = Socks5Server::<AcceptAuthentication>::bind(self.addr)
            .await
            .context("failed to bind socks5 server")?
            .with_config(config);

        let ip = self.addr.ip();
        let stream = stream::try_unfold(server, move |server| async move {
            let mut incoming = server.incoming();

            if let Some(res) = incoming.next().await {
                let mut socks5 = res.context("failed to accept socks5 connection")?;
                socks5.set_reply_ip(ip);
                println!("accept socks5 connection");
                let socks5 = socks5
                    .upgrade_to_socks5()
                    .await
                    .context("failed to upgrade to socks5")?;
                println!("upgrade to socks5");
                let target_addr: Target = socks5
                    .target_addr()
                    .ok_or(Error::new(ErrorKind::GetTargetFailed, "get target failed"))?
                    .into();
                drop(incoming);

                return Ok(Some((Incoming::new(target_addr, socks5), server)));
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

impl From<&TargetAddr> for crate::Target {
    fn from(addr: &TargetAddr) -> Self {
        match addr {
            TargetAddr::Ip(ip) => Self::Ip(*ip),
            TargetAddr::Domain(domain, port) => Self::Domain(domain.clone(), *port),
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
