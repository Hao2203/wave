// #![allow(unused_imports)]
use super::*;
use crate::{Address, AddressFromStrErr};
use bytes::{Bytes, BytesMut};
use derive_more::derive::{Display, From};
use std::{collections::VecDeque, net::SocketAddr};
use types::*;

pub mod codec;
#[cfg(test)]
mod tests;
pub mod types;

#[derive(Debug, PartialEq, Eq)]
pub struct Socks5 {
    transmits: VecDeque<Transmit>,
    events: VecDeque<Event>,
    status: Status,
    tcp_bind: SocketAddr,
    udp_bind: Option<SocketAddr>,
    relay_server: Option<Address>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Init,
    Handshake,
    Relay {
        target: Address,
        status: Option<ConnectedStatus>,
        source: Address,
    },
    Close,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    BufferIncomplete,
    Handshake,
    ConnectToTarget { target: Address },
    Close { reason: Option<Error> },
    Error(Error),
}

impl Socks5 {
    pub fn new(tcp_bind: SocketAddr) -> Self {
        Socks5 {
            transmits: VecDeque::new(),
            events: VecDeque::new(),
            status: Status::Init,
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

    pub fn poll_transmit(&mut self) -> Option<Transmit> {
        self.transmits.pop_front()
    }

    pub fn poll_event(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    pub fn handle_input(
        &mut self,
        protocol: Protocol,
        source: Address,
        input: &mut BytesMut,
    ) -> usize {
        let len = input.len();
        let res = self.process_input(Input {
            protocol,
            source,
            data: input,
        });
        let len = len - input.len();
        match res {
            Ok(Some(transmit)) => {
                self.transmits.push_back(transmit);
            }
            Err(Error::LengthNotEnough { .. }) => return 0,
            Err(e) => {
                self.events.push_back(Event::Error(e));
            }
            _ => (),
        }
        len
    }

    fn process_input(&mut self, input: Input<'_>) -> Result<Option<Transmit>, Error> {
        let data = input.data.as_ref();
        match &self.status {
            Status::Init => {
                let request = codec::decode_consult_request(data)?;
                let res = self.process_consult_request(request)?;
                let to = if let Address::Ip(ip) = input.source {
                    ip
                } else {
                    return Err(Error::UnexpectedAddressType {
                        address: input.source,
                    });
                };
                let res = Transmit {
                    proto: Protocol::Tcp,
                    local: self.tcp_bind(),
                    to: Address::Ip(to),
                    data: codec::encode_consult_response(res),
                };
                self.set_status(Status::Handshake);
                Ok(Some(res))
            }
            Status::Handshake => {
                let request = codec::decode_connect_request(data)?;
                self.set_status(Status::Relay {
                    target: request.target.clone(),
                    status: None,
                    source: input.source,
                });
                self.events.push_back(Event::ConnectToTarget {
                    target: request.target,
                });
                Ok(None)
            }
            Status::Relay {
                target,
                status,
                source,
            } => {
                if status.is_none() {
                    return Ok(None);
                }
                let data = input.data.split().freeze();
                let to = if input.source == *source {
                    target.clone()
                } else {
                    source.clone()
                };
                let transmit = Transmit {
                    proto: Protocol::Tcp,
                    local: self.tcp_bind(),
                    to,
                    data,
                };
                Ok(Some(transmit))
            }
            Status::Close => Ok(None),
        }
    }

    pub fn connect_with_status(&mut self, status: ConnectedStatus) {
        let res = self.connect_with_status_inner(status);
        if let Some(res) = res {
            self.transmits.push_back(res);
        }
    }

    fn connect_with_status_inner(&mut self, status: ConnectedStatus) -> Option<Transmit> {
        if let Status::Relay {
            target,
            status: this_status,
            source,
        } = &self.status
        {
            if this_status.is_some() {
                return None;
            }
            let to = source.clone();
            let data = match status {
                ConnectedStatus::Succeeded => {
                    let res = ConnectResponse {
                        status: ConnectedStatus::Succeeded,
                        bind_address: self.tcp_bind().into(),
                    };
                    let data = codec::encode_connect_response(res);
                    self.set_status(Status::Relay {
                        target: target.clone(),
                        status: Some(ConnectedStatus::Succeeded),
                        source: source.clone(),
                    });
                    data
                }
                _ => {
                    let res = ConnectResponse {
                        status,
                        bind_address: target.clone(),
                    };
                    let data = codec::encode_connect_response(res);
                    self.events
                        .push_back(Event::Error(Error::ConnectToTargetFailed {
                            target: target.clone(),
                            status,
                        }));
                    self.set_status(Status::Close);
                    data
                }
            };
            let transmit = Transmit {
                proto: Protocol::Tcp,
                local: self.tcp_bind(),
                to,
                data,
            };
            return Some(transmit);
        }
        None
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
pub struct Input<'a> {
    pub protocol: Protocol,
    pub source: Address,
    pub data: &'a mut BytesMut,
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
