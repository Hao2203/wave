use super::*;
use bytes::Bytes;

pub mod socks5;

pub struct Receive {
    pub proto: Protocol,
    pub local: SocketAddr,
    pub source: SocketAddr,
    pub data: Bytes,
}

#[derive(Debug)]
pub enum Protocol {
    Tcp,
    Udp,
}
