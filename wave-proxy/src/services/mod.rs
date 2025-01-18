use std::time::Instant;

use super::*;
use bytes::Bytes;

pub mod socks5;

pub trait Proxy {
    fn poll_output(&mut self, now: Instant, input: Input) -> Result<Output>;
}

pub enum Output {
    Pending,
    Consult(Transmit),
    Connect(Connect),
    Relay(Relay),
    Close,
}

#[derive(Debug)]
pub struct Bind {
    pub local: SocketAddr,
    pub mode: Protocol,
}

pub struct Connect {
    pub bind: Bind,
    pub remote_target: Target,
    pub transmit: Transmit,
}

pub struct Transmit {
    pub mode: Protocol,
    pub local: SocketAddr,
    pub to: SocketAddr,
    pub data: Bytes,
}

pub struct Relay {
    pub target: Target,
    pub data: Bytes,
}

pub enum Input {
    Receive(Receive),
}

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
