// #![allow(unused_imports)]
use super::*;
use error::Context;
use fast_socks5::{
    consts,
    server::{AcceptAuthentication, Config, Socks5Socket},
    util::target_addr::TargetAddr,
    ReplyError, Socks5Command, SocksError,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::io::AsyncWriteExt;

pub struct Socks5 {}

#[async_trait::async_trait]
impl<T: Connection + Unpin> Proxy<T> for Socks5 {
    async fn serve<'a>(&self, incoming: Incoming<T>) -> Result<ProxyStatus<'a, T>>
    where
        T: 'a,
    {
        let local_addr = incoming.local_addr;

        let mut config = Config::<AcceptAuthentication>::default();
        config.set_execute_command(false);
        let mut socks5 = Socks5Socket::new(incoming, Arc::new(config))
            .upgrade_to_socks5()
            .await
            .context("failed to upgrade to socks5")?;
        // println!("socks5 upgrade success");

        let target: Target = socks5
            .target_addr()
            .ok_or(Error::new(ErrorInner::GetTargetFailed, "get target failed"))?
            .into();
        match socks5.cmd() {
            None => Ok(ProxyStatus::Continue(socks5.into_inner().conn)),
            Some(cmd) => match cmd {
                Socks5Command::TCPConnect => {
                    reply_success(&mut socks5, local_addr).await?;

                    let info = ProxyHandler {
                        proxy_mode: "socks5".into(),
                        target,
                        tunnel: Box::pin(socks5),
                    };
                    Ok(ProxyStatus::Success(info))
                }
                Socks5Command::UDPAssociate => {
                    todo!()
                }
                _ => Err(Error::new(
                    ErrorInner::UnSupportedProxyProtocol,
                    "parse command failed",
                )),
            },
        }
    }
}

async fn reply_success(io: &mut (impl Connection + Unpin), addr: SocketAddr) -> Result<()> {
    let reply = new_reply(&ReplyError::Succeeded, addr);
    reply_to(io, reply).await
}

async fn reply_to(io: &mut (impl Connection + Unpin), reply: impl AsRef<[u8]>) -> Result<()> {
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

impl From<SocksError> for ErrorInner {
    fn from(value: SocksError) -> Self {
        type E = SocksError;
        match value {
            E::Io(e) => e.into(),
            E::InvalidHeader {
                expected: _,
                found: _,
            }
            | E::UnsupportedSocksVersion(_) => ErrorInner::UnSupportedProxyProtocol,
            _ => ErrorInner::Unexpected,
        }
    }
}
