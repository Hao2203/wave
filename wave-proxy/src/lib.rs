use bytes::Bytes;
use derive_more::derive::{Display, From};
use protocol::socks5::{self, NoAuthHandshake};
use std::{net::SocketAddr, str::FromStr, sync::Arc};

pub mod protocol;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Proxy {
    local: SocketAddr,
}

impl Proxy {
    pub fn new(local: SocketAddr) -> Self {
        Self { local }
    }

    pub fn socks5_addr(&self) -> SocketAddr {
        self.local
    }

    pub fn accept_socks5(
        &self,
        client: SocketAddr,
        request: socks5::HandshakeRequest,
    ) -> (Transmit, Result<socks5::Connecting, socks5::Error>) {
        NoAuthHandshake::new(self.local, client).handshake(request)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transmit {
    pub proto: Protocol,
    pub local: SocketAddr,
    pub to: Address,
    pub data: Bytes,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    #[display("TCP")]
    Tcp,
    #[display("UDP")]
    Udp,
}

#[derive(Debug, Display, From, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Address {
    #[from]
    Ip(SocketAddr),
    #[display("{_0}:{_1}")]
    Domain(Arc<str>, u16),
}

impl FromStr for Address {
    type Err = AddressFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(ip) = s.parse() {
            return Ok(Address::Ip(ip));
        }
        if let Some(colon) = s.find(':') {
            let domain = s[0..colon].to_string();
            let port = s[colon + 1..].parse::<u16>()?;
            return Ok(Address::Domain(Arc::from(domain), port));
        }
        Err(AddressFromStrErr::Other(s.to_string()))
    }
}

#[derive(Debug, From, Display, PartialEq, Eq, derive_more::Error)]
#[error(ignore)]
pub enum AddressFromStrErr {
    #[from]
    IpParseError(std::net::AddrParseError),
    #[from]
    IntParseError(std::num::ParseIntError),
    #[display("Parse address failed: {_0}")]
    Other(String),
}
