use bytes::{Buf, BufMut, Bytes, BytesMut};
use derive_more::{Display, Error, From};
pub use error::Error;
use std::{borrow::Cow, sync::Arc};

pub mod error;
pub mod server;
#[cfg(test)]
mod test;

pub struct Connection {
    node_id: NodeId,
    subdomain: Arc<str>,
    port: u16,
}

impl Connection {
    pub fn connect(domain: &str, port: u16) -> Result<(Bytes, Connection), Error> {
        let mut fragment = domain.split('.').collect::<Vec<_>>();
        let node_id: NodeId = fragment.pop().unwrap().parse()?;
        let subdomain = Arc::from(fragment.join("."));

        let data = WavePacket::new(port, Arc::clone(&subdomain)).encode();

        Ok((data, Connection {
            node_id,
            subdomain,
            port,
        }))
    }

    pub fn accept(node_id: NodeId, packet: WavePacket) -> Connection {
        Connection {
            node_id,
            subdomain: packet.subdomain,
            port: packet.port,
        }
    }

    pub fn send<'a>(&self, data: &'a [u8]) -> Cow<'a, [u8]> {
        Cow::Borrowed(data)
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn subdomain(&self) -> Arc<str> {
        self.subdomain.clone()
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

pub struct WavePacket {
    pub port: u16,
    pub subdomain: Arc<str>,
}

impl WavePacket {
    pub fn new(port: u16, subdomain: Arc<str>) -> Self {
        Self { port, subdomain }
    }

    pub fn decode(data: &mut BytesMut) -> Result<Option<Self>, WavePacketDecodeError> {
        if data.remaining() < 6 {
            return Ok(None);
        }

        let port = data.get_u16();
        let subdomain_len = data.get_u32();

        if data.remaining() < subdomain_len as usize {
            return Ok(None);
        }

        let subdomain = data.split_to(subdomain_len as usize);
        let subdomain = Arc::from(std::str::from_utf8(subdomain.as_ref())?);

        Ok(Some(WavePacket { port, subdomain }))
    }

    pub fn encode(self) -> Bytes {
        let mut buf = BytesMut::with_capacity(2 + 4 + self.subdomain.len());
        buf.put_u16(self.port);
        buf.put_u32(self.subdomain.len() as u32);
        buf.put(self.subdomain.as_bytes());
        buf.freeze()
    }
}

#[derive(Debug, Display, From, Error)]
pub enum WavePacketDecodeError {
    Utf8Error(std::str::Utf8Error),
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
