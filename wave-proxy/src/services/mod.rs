use bytes::Bytes;

use super::*;

pub mod socks5;

pub trait Proxy {
    fn poll_target(&mut self, local_addr: SocketAddr, packet: &[u8]) -> Option<Target>;
}
