#![allow(unused)]
use crate::{error::Context, Error, ErrorKind, Incoming, Info, Io, Proxy, Result, Target};
use fast_socks5::{
    server::{AcceptAuthentication, Config, Socks5Server, Socks5Socket},
    util::target_addr::TargetAddr,
    SocksError,
};
use futures_lite::{
    stream::{self, Boxed},
    StreamExt,
};
use std::{net::SocketAddr, sync::Arc};

pub struct Socks5 {}

#[async_trait::async_trait]
impl Proxy for Socks5 {
    async fn proxy(&self, conn: Box<dyn Io>) -> Result<(Info, Box<dyn Io>)> {
        let mut config = Config::<AcceptAuthentication>::default();
        config.set_execute_command(false);
        let socks5 = Socks5Socket::new(conn, Arc::new(config));
        let socks5 = socks5
            .upgrade_to_socks5()
            .await
            .context("failed to upgrade to socks5")?;
        println!("upgrade to socks5");
        let target_addr: Target = socks5
            .target_addr()
            .ok_or(Error::new(ErrorKind::GetTargetFailed, "get target failed"))?
            .into();
        let info = Info {
            target: target_addr,
        };
        Ok((info, Box::new(socks5)))
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
