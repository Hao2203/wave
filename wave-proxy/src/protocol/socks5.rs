// #![allow(unused_imports)]
use super::*;
use crate::{Address, AddressFromStrErr};
use bytes::Bytes;
use derive_more::derive::{Display, From};
use std::{net::SocketAddr, sync::Arc};
use types::*;

// pub mod codec;
#[cfg(test)]
mod tests;
pub mod types;

pub struct NoAuthHandshake {
    tcp_bind: SocketAddr,
    client: SocketAddr,
}

impl NoAuthHandshake {
    pub fn new(tcp_bind: SocketAddr, client: SocketAddr) -> Self {
        NoAuthHandshake { tcp_bind, client }
    }
    pub fn handshake(self, request: HandshakeRequest) -> (Transmit, Result<Connecting, Error>) {
        let response = if !request.methods.iter().any(|x| *x == AuthMethod::None) {
            HandshakeResponse(AuthMethod::NotAcceptable)
        } else {
            HandshakeResponse(AuthMethod::None)
        };
        let data = response.encode();
        let transmit = Transmit {
            proto: Protocol::Tcp,
            local: self.tcp_bind,
            to: Address::Ip(self.client),
            data,
        };

        let res = if response.is_acceptable() {
            Ok(Connecting {
                tcp_bind: self.tcp_bind,
                client: self.client,
            })
        } else {
            Err(Error::UnSupportedMethods {
                methods: request.methods,
            })
        };
        (transmit, res)
    }
}

pub struct Connecting {
    tcp_bind: SocketAddr,
    client: SocketAddr,
}

impl Connecting {
    pub fn connect(
        self,
        request: ConnectRequest,
        status: ConnectedStatus,
    ) -> (Transmit, Result<Relay, Error>) {
        let target = request.target;
        let bytes = ConnectResponse {
            status,
            bind_address: self.tcp_bind.into(),
        }
        .encode();
        let transmit = Transmit {
            proto: Protocol::Tcp,
            local: self.tcp_bind,
            to: self.client.into(),
            data: bytes,
        };
        let res = if status == ConnectedStatus::Succeeded {
            Ok(Relay {
                target,
                client: self.client,
                tcp_bind: self.tcp_bind,
            })
        } else {
            Err(Error::ConnectToTargetFailed { target, status })
        };
        (transmit, res)
    }
}

#[derive(Debug)]
pub struct Relay {
    tcp_bind: SocketAddr,
    client: SocketAddr,
    target: Address,
}

impl Relay {
    pub fn relay(&mut self, from: Address, data: Bytes) -> Transmit {
        let to = if from == self.target {
            self.client.into()
        } else {
            self.target.clone()
        };
        Transmit {
            proto: Protocol::Tcp,
            local: self.tcp_bind,
            to,
            data,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transmit {
    pub proto: Protocol,
    pub local: SocketAddr,
    pub to: Address,
    pub data: Bytes,
}

#[derive(Debug, Display, From, PartialEq, Eq, derive_more::Error)]
pub enum Error {
    #[display("Connect to target failed: {target}, status: {status}")]
    ConnectToTargetFailed {
        target: Address,
        status: ConnectedStatus,
    },
    #[display("Unexpected protocol: {protocol}, source_address: {source_address}")]
    UnexpectedProtocol {
        protocol: Protocol,
        source_address: Address,
    },
    #[display("Unexpected address type: {address}")]
    UnexpectedAddressType { address: Address },
    #[display("UnSupportedMethod: {methods:?}")]
    UnSupportedMethods { methods: Arc<[AuthMethod]> },
    #[display("Invalid version: {version}")]
    InvalidVersion { version: u8 },
    #[display("Invalid method: {method}")]
    InvalidMethod { method: u8 },
    #[display("Invalid command: {command}")]
    InvalidCommand { command: u8 },
    #[display("Invalid address type: {addr_type}")]
    InvalidAddrType { addr_type: u8 },
    #[from]
    FromUtf8Error(std::string::FromUtf8Error),
    #[from]
    AddressParseError(AddressFromStrErr),
}
