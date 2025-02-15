pub use connection::{Connection, WavePacket};
use derive_more::{AsRef, Display, Error, From};
pub use error::Error;
use serde::{Deserialize, Serialize};
pub use server::Server;
use std::{net::IpAddr, ops::Deref, str::FromStr, sync::Arc};

pub mod connection;
pub mod error;
pub mod router;
pub mod server;
#[cfg(test)]
mod test;

#[derive(Debug, Clone, Display, From)]
pub enum Host {
    Ip(IpAddr),
    Domain(Arc<str>),
}

impl Host {
    pub const MAX_LEN: usize = 255;
}

impl FromStr for Host {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > Host::MAX_LEN {
            return Err(Error::DomainOverflow(Arc::from(s)));
        }
        if let Ok(ip) = s.parse() {
            Ok(Host::Ip(ip))
        } else {
            Ok(Host::Domain(Arc::from(s)))
        }
    }
}

#[derive(Debug, Clone, Display, AsRef, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Subdomain(Arc<str>);

impl Subdomain {
    pub const MAX_LEN: usize = 255;

    pub fn new(subdomain: Arc<str>) -> Result<Self, Error> {
        if subdomain.len() > Self::MAX_LEN {
            Err(Error::SubdomainOverflow(subdomain))
        } else {
            Ok(Subdomain(subdomain))
        }
    }
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl FromStr for Subdomain {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(Arc::from(s))
    }
}

impl Deref for Subdomain {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub iroh::PublicKey);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let encoder = data_encoding::BASE32_DNSSEC;
        let bs32 = encoder.encode_display(self.0.as_bytes());
        write!(f, "{}", bs32)
    }
}

#[derive(Debug, Display, From, Error)]
pub enum NodeIdParsingError {
    Decode(data_encoding::DecodeError),
    Key(ed25519_dalek::SignatureError),
}

impl std::str::FromStr for NodeId {
    type Err = NodeIdParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = data_encoding::BASE32_DNSSEC.decode(s.as_bytes())?;
        let public_key = iroh::PublicKey::try_from(bytes.as_slice())?;
        Ok(NodeId(public_key))
    }
}
