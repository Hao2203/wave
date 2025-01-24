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
pub struct Socks5Proxy {
    recvs: VecDeque<Input>,
    status: Status,
    tcp_bind: SocketAddr,
    udp_bind: Option<SocketAddr>,
    relay_server: Option<Address>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Consulting,
    Connecting,
    Relaying { target: Address },
}

impl Socks5Proxy {
    pub fn new(tcp_bind: SocketAddr) -> Self {
        Socks5Proxy {
            recvs: VecDeque::new(),
            status: Status::Consulting,
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
            Status::Consulting => {
                let request = codec::decode_consult_request(&mut input.data)?;
                let res = self.process_consult_request(request)?;
                let res = Output::Transmit(Transmit {
                    proto: Protocol::Tcp,
                    local: self.tcp_bind,
                    to: input.source,
                    data: codec::encode_consult_response(res),
                });
                self.set_status(Status::Connecting);
                Ok(res)
            }
            Status::Connecting => {
                let request = codec::decode_connect_request(&mut input.data)?;
                let bind_address = self.tcp_bind;
                let connect = Connect {
                    proxy: self,
                    protocol: Protocol::Tcp,
                    target: request.target,
                    source: input.source,
                    bind_address,
                };
                Ok(Output::Connect(connect))
            }
            Status::Relaying { target } => {
                let transmit = Transmit {
                    proto: input.protocol(),
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
    protocol: Protocol,
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

    pub fn protocol(&self) -> Protocol {
        self.protocol
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Output<'a> {
    Pending,
    Transmit(Transmit),
    Connect(Connect<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Transmit {
    pub proto: Protocol,
    pub local: SocketAddr,
    pub to: Address,
    pub data: Bytes,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Connect<'a> {
    proxy: &'a mut Socks5Proxy,
    pub protocol: Protocol,
    pub target: Address,
    pub source: Address,
    pub bind_address: SocketAddr,
}

impl Connect<'_> {
    pub fn target(&self) -> &Address {
        &self.target
    }

    pub fn connected_success(self) -> Transmit {
        let res = ConnectResponse {
            status: ConnectedStatus::Succeeded,
            bind_address: self.bind_address.into(),
        };
        let data = codec::encode_connect_response(res);
        self.proxy.set_status(Status::Relaying {
            target: self.target,
        });
        Transmit {
            proto: Protocol::Tcp,
            local: self.bind_address,
            to: self.source,
            data,
        }
    }

    pub fn connected_failed(self, status: ConnectedStatus) -> Transmit {
        let res = ConnectResponse {
            status,
            bind_address: self.target,
        };
        let data = codec::encode_connect_response(res);
        self.proxy.set_status(Status::Consulting);
        Transmit {
            proto: Protocol::Tcp,
            local: self.bind_address,
            to: self.source,
            data,
        }
    }
}
