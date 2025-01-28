use derive_more::derive::{Display, From};
use std::{net::SocketAddr, str::FromStr, sync::Arc};

pub mod protocol;
#[cfg(test)]
mod tests;

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
