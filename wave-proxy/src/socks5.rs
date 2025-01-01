#![allow(unused_imports)]
use crate::{
    error::Context, Connection, Error, ErrorKind, Proxy, ProxyCtx, ProxyInfo, Result, Target,
};
use fast_socks5::{
    consts,
    server::{AcceptAuthentication, Config, Socks5Server, Socks5Socket},
    util::target_addr::TargetAddr,
    ReplyError, Socks5Command, SocksError,
};
use futures_lite::{
    stream::{self, Boxed},
    StreamExt,
};
use std::{
    net::SocketAddr,
    pin::{pin, Pin},
    sync::Arc,
};
use tokio::io::AsyncWriteExt;

pub struct Socks5 {}

#[async_trait::async_trait]
impl Proxy for Socks5 {
    const ROUTE_SIZE: usize = 20;

    async fn proxy_incoming(
        &self,
        ctx: &mut dyn ProxyCtx,
        incoming: &mut (dyn Connection + Unpin),
    ) -> Result<()> {
        let mut config = Config::<AcceptAuthentication>::default();
        config.set_execute_command(false);
        let mut socks5 = Socks5Socket::new(incoming, Arc::new(config))
            .upgrade_to_socks5()
            .await
            .context("failed to upgrade to socks5")?;
        // println!("socks5 upgrade success");
        let target: Target = socks5
            .target_addr()
            .ok_or(Error::new(ErrorKind::GetTargetFailed, "get target failed"))?
            .into();

        let info = ProxyInfo {
            proxy_mode: "socks5".into(),
            target,
        };

        if let Err(e) = ctx.upstream_session(&info).await {
            let reply = new_reply(&ReplyError::ConnectionRefused, ctx.local_addr());
            reply_to(&mut socks5, reply).await?;
            return Err(e);
        };

        let command = socks5.cmd();
        match command {
            None => Err(Error::new(
                ErrorKind::UnSupportedProxyProtocol,
                "command is none",
            )),
            Some(cmd) => match cmd {
                Socks5Command::TCPConnect => {
                    reply_success(&mut socks5, ctx.local_addr()).await?;

                    ctx.process_tunnel(&mut socks5).await?;

                    Ok(())
                }
                Socks5Command::UDPAssociate => {
                    todo!()
                }
                _ => Err(Error::new(
                    ErrorKind::UnSupportedProxyProtocol,
                    "parse command failed",
                )),
            },
        }
    }
}

async fn reply_success(io: &mut (dyn Connection + Unpin), addr: SocketAddr) -> Result<()> {
    let reply = new_reply(&ReplyError::Succeeded, addr);
    reply_to(io, reply).await
}

async fn reply_to(io: &mut (dyn Connection + Unpin), reply: impl AsRef<[u8]>) -> Result<()> {
    io.write_all(reply.as_ref())
        .await
        .context("failed to write reply")
}

/// Generate reply code according to the RFC.
fn new_reply(error: &ReplyError, sock_addr: SocketAddr) -> Vec<u8> {
    let (addr_type, mut ip_oct, mut port) = match sock_addr {
        SocketAddr::V4(sock) => (
            consts::SOCKS5_ADDR_TYPE_IPV4,
            sock.ip().octets().to_vec(),
            sock.port().to_be_bytes().to_vec(),
        ),
        SocketAddr::V6(sock) => (
            consts::SOCKS5_ADDR_TYPE_IPV6,
            sock.ip().octets().to_vec(),
            sock.port().to_be_bytes().to_vec(),
        ),
    };

    let mut reply = vec![
        consts::SOCKS5_VERSION,
        error.as_u8(), // transform the error into byte code
        0x00,          // reserved
        addr_type,     // address type (ipv4, v6, domain)
    ];
    reply.append(&mut ip_oct);
    reply.append(&mut port);

    reply
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
