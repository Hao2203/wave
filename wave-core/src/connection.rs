use crate::{Error, NodeId, Subdomain};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use derive_more::{Display, From};
use std::{borrow::Cow, sync::Arc};

pub struct Connection {
    node_id: NodeId,
    subdomain: Subdomain,
    port: u16,
}

impl Connection {
    pub fn connect(domain: &str, port: u16) -> Result<(Bytes, Connection), Error> {
        let mut fragment = domain.split('.').collect::<Vec<_>>();
        let node_id: NodeId = fragment.pop().unwrap().parse()?;
        let subdomain = Arc::from(fragment.join("."));
        let subdomain = Subdomain::new(subdomain)?;

        let data = WavePacket::new(port, subdomain.clone()).encode();

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

    pub fn subdomain(&self) -> Subdomain {
        self.subdomain.clone()
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

pub struct WavePacket {
    pub port: u16,
    pub subdomain: Subdomain,
}

impl WavePacket {
    pub fn new(port: u16, subdomain: Subdomain) -> Self {
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
        } else if subdomain_len > Subdomain::MAX_LEN as u32 {
            return Err(WavePacketDecodeError::SubdomainOverflow);
        }

        let subdomain = data.split_to(subdomain_len as usize);
        let subdomain = Arc::from(std::str::from_utf8(subdomain.as_ref())?);
        let subdomain = Subdomain::new(subdomain).unwrap();

        Ok(Some(WavePacket { port, subdomain }))
    }

    pub fn encode(self) -> Bytes {
        let mut buf = BytesMut::with_capacity(2 + 4 + self.subdomain.as_str().len());
        buf.put_u16(self.port);
        buf.put_u32(self.subdomain.as_str().len() as u32);
        buf.put(self.subdomain.as_str().as_bytes());
        buf.freeze()
    }
}

#[derive(Debug, Display, From, Error)]
pub enum WavePacketDecodeError {
    Utf8Error(std::str::Utf8Error),
    #[display("Subdomain overflow")]
    SubdomainOverflow,
}
