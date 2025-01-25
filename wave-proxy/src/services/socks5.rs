// #![allow(unused_imports)]
use super::*;
use crate::{Address, Error, ErrorKind, Result};
use bytes::Bytes;
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

    pub fn poll_output(&mut self) -> Result<Output> {
        self.recvs
            .pop_front()
            .map(|input| self.process_input(input))
            .unwrap_or(Ok(Output::Pending))
    }

    pub fn input(&mut self, input: Input) {
        self.recvs.push_back(input);
    }

    fn process_input(&mut self, mut input: Input) -> Result<Output> {
        match &self.status {
            Status::Handshake => {
                let request = codec::decode_consult_request(&mut input.data)?;
                let res = self.process_consult_request(request)?;
                let to = if let Address::Ip(ip) = input.source {
                    ip
                } else {
                    return Err(Error::message(
                        ErrorKind::UnSupportedProxyProtocol,
                        "Invalid socks5 request",
                    ));
                };
                let res = Output::Handshake(Handshake {
                    to,
                    data: codec::encode_consult_response(res),
                });
                self.set_status(Status::Connecting);
                Ok(res)
            }
            Status::Connecting => {
                let request = codec::decode_connect_request(&mut input.data)?;
                let bind_address = self.tcp_bind;
                let connect = TcpConnect {
                    proxy: self,
                    target: request.target,
                    source: input.source,
                    bind_address,
                    data: (),
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
                Ok(Output::Transmit(transmit))
            }
        }
    }

    fn process_consult_request(&self, request: ConsultRequest) -> Result<ConsultResponse> {
        if !request.methods.contains(&AuthMethod::None) {
            return Err(Error::message(
                ErrorKind::UnSupportedProxyProtocol,
                "Invalid socks5 request",
            ));
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
    Handshake(Handshake),
    Transmit(Transmit),
    TcpConnect(TcpConnect<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Handshake {
    pub to: SocketAddr,
    pub data: Bytes,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Transmit {
    pub proto: Protocol,
    pub local: SocketAddr,
    pub to: Address,
    pub data: Bytes,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TcpConnect<'a, T = ()> {
    proxy: &'a mut Socks5,
    pub target: Address,
    pub source: Address,
    pub bind_address: SocketAddr,
    pub data: T,
}

impl<'a> TcpConnect<'a> {
    pub fn connected_success(self) -> TcpConnect<'a, Bytes> {
        self.connect_with_status(ConnectedStatus::Succeeded)
    }

    pub fn connect_with_status(self, status: ConnectedStatus) -> TcpConnect<'a, Bytes> {
        match status {
            ConnectedStatus::Succeeded => {
                let res = ConnectResponse {
                    status: ConnectedStatus::Succeeded,
                    bind_address: self.bind_address.into(),
                };
                let data = codec::encode_connect_response(res);
                self.proxy.set_status(Status::Relay {
                    target: self.target.clone(),
                });
                TcpConnect {
                    proxy: self.proxy,
                    target: self.target,
                    source: self.source,
                    bind_address: self.bind_address,
                    data,
                }
            }
            _ => {
                let res = ConnectResponse {
                    status,
                    bind_address: self.target.clone(),
                };
                let data = codec::encode_connect_response(res);
                self.proxy.set_status(Status::Handshake);
                TcpConnect {
                    proxy: self.proxy,
                    target: self.target,
                    source: self.source,
                    bind_address: self.bind_address,
                    data,
                }
            }
        }
    }
}
