// #![allow(unused_imports)]
use super::*;
use crate::{Address, AddressFromStrErr};
use bytes::Bytes;
use derive_more::derive::{Display, From};
use std::{collections::VecDeque, net::SocketAddr};
use types::*;

pub mod codec;
#[cfg(test)]
mod tests;
pub mod types;

#[derive(Debug, PartialEq, Eq)]
pub struct Socks5 {
    recvs: VecDeque<Input>,
    status: Status,
    tcp_bind: SocketAddr,
    udp_bind: Option<SocketAddr>,
    relay_server: Option<Address>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Handshake,
    Connecting,
    Relay { target: Address },
}

impl Socks5 {
    pub fn new(tcp_bind: SocketAddr) -> Self {
        Socks5 {
            recvs: VecDeque::new(),
            status: Status::Handshake,
            tcp_bind,
            udp_bind: None,
            relay_server: None,
        }
    }

    pub fn tcp_bind(&self) -> SocketAddr {
        self.tcp_bind
    }

    pub fn udp_bind(&self) -> Option<SocketAddr> {
        self.udp_bind
    }

    pub fn relay_server(&self) -> Option<Address> {
        self.relay_server.clone()
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn poll_output(&mut self) -> Result<Output, Error> {
        self.recvs
            .pop_front()
            .map(|input| self.process_input(input))
            .unwrap_or(Ok(Output::Pending))
    }

    pub fn input(&mut self, input: Input) {
        self.recvs.push_back(input);
    }

    fn process_input(&mut self, mut input: Input) -> Result<Output, Error> {
        match &self.status {
            Status::Handshake => {
                let request = codec::decode_consult_request(&mut input.data)?;
                let res = self.process_consult_request(request)?;
                let to = if let Address::Ip(ip) = input.source {
                    ip
                } else {
                    return Err(Error::UnexpectedAddressType {
                        address: input.source,
                    });
                };
                let res = Output::Handshake(Transmit {
                    proto: Protocol::Tcp,
                    local: self.tcp_bind(),
                    to: Address::Ip(to),
                    data: codec::encode_consult_response(res),
                });
                self.set_status(Status::Connecting);
                Ok(res)
            }
            Status::Connecting => {
                let request = codec::decode_connect_request(&mut input.data)?;
                let source = if let Address::Ip(ip) = input.source {
                    ip
                } else {
                    return Err(Error::UnexpectedAddressType {
                        address: input.source,
                    });
                };
                let connect = TcpConnect {
                    proxy: self,
                    target: request.target,
                    source,
                };
                Ok(Output::TcpConnect(connect))
            }
            Status::Relay { target } => {
                let transmit = Transmit {
                    proto: input.protocol,
                    local: self.tcp_bind(),
                    to: target.clone(),
                    data: input.data,
                };
                Ok(Output::Relay(transmit))
            }
        }
    }

    fn process_consult_request(&self, request: ConsultRequest) -> Result<ConsultResponse, Error> {
        if !request.methods.contains(&AuthMethod::None) {
            return Err(Error::UnSupportedMethods {
                methods: request.methods,
            });
        }
        Ok(ConsultResponse(AuthMethod::None))
    }

    fn set_status(&mut self, status: Status) {
        self.status = status;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Input {
    pub protocol: Protocol,
    pub source: Address,
    pub data: Bytes,
}

impl Input {
    pub fn new_tcp(source: Address, data: Bytes) -> Self {
        Input {
            protocol: Protocol::Tcp,
            source,
            data,
        }
    }

    pub fn new_udp(source: Address, data: Bytes) -> Self {
        Input {
            protocol: Protocol::Udp,
            source,
            data,
        }
    }

    pub fn is_tcp(&self) -> bool {
        matches!(self.protocol, Protocol::Tcp)
    }

    pub fn is_udp(&self) -> bool {
        matches!(self.protocol, Protocol::Udp)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Output<'a> {
    Pending,
    Handshake(Transmit),
    TcpConnect(TcpConnect<'a>),
    Relay(Transmit),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transmit {
    pub proto: Protocol,
    pub local: SocketAddr,
    pub to: Address,
    pub data: Bytes,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TcpConnect<'a> {
    proxy: &'a mut Socks5,
    target: Address,
    source: SocketAddr,
}

impl TcpConnect<'_> {
    pub fn target(&self) -> Address {
        self.target.clone()
    }

    pub fn connected_success(&mut self) -> Transmit {
        self.connect_with_status(ConnectedStatus::Succeeded)
    }

    pub fn connect_with_status(&mut self, status: ConnectedStatus) -> Transmit {
        let data = match status {
            ConnectedStatus::Succeeded => {
                let res = ConnectResponse {
                    status: ConnectedStatus::Succeeded,
                    bind_address: self.proxy.tcp_bind().into(),
                };
                let data = codec::encode_connect_response(res);
                self.proxy.set_status(Status::Relay {
                    target: self.target(),
                });
                data
            }
            _ => {
                let res = ConnectResponse {
                    status,
                    bind_address: self.target(),
                };
                let data = codec::encode_connect_response(res);
                self.proxy.set_status(Status::Handshake);
                data
            }
        };
        Transmit {
            proto: Protocol::Tcp,
            local: self.proxy.tcp_bind(),
            to: Address::Ip(self.source),
            data,
        }
    }
}

#[derive(Debug, Display, From, derive_more::Error)]
pub enum Error {
    #[display("Unexpected protocol: {protocol}, source_address: {source_address}")]
    UnexpectedProtocol {
        protocol: Protocol,
        source_address: Address,
    },
    #[display("Unexpected address type: {address}")]
    UnexpectedAddressType { address: Address },
    #[display("UnSupportedMethod: {methods:?}")]
    UnSupportedMethods { methods: Vec<AuthMethod> },
    #[display("Length not enough: {len}")]
    LengthNotEnough { len: usize },
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
