// #![allow(unused_imports)]
use super::*;
use crate::{Address, Error, ErrorKind, Result};
use bytes::Bytes;
use std::{collections::VecDeque, net::SocketAddr};
use types::*;

pub mod codec;
pub mod types;

#[derive(Debug, PartialEq, Eq)]
pub struct Socks5Proxy {
    recvs: VecDeque<Input>,
    status: Status,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Consulting,
    Connecting,
    Relaying { target: Address },
}

impl Socks5Proxy {
    pub fn new() -> Self {
        Socks5Proxy {
            recvs: VecDeque::new(),
            status: Status::Consulting,
        }
    }

    pub fn poll_output(&mut self) -> Result<Output> {
        self.recvs
            .pop_front()
            .map(|input| match input {
                Input::Receive(receive) => self.process_receive(receive),
            })
            .unwrap_or(Ok(Output::Pending))
    }

    pub fn input(&mut self, input: Input) {
        self.recvs.push_back(input);
    }

    fn process_receive(&mut self, mut receive: Receive) -> Result<Output> {
        match &self.status {
            Status::Consulting => {
                let request = codec::decode_consult_request(&mut receive.data)?;
                let res = self.process_consult_request(request)?;
                let res = Output::Transmit(Transmit {
                    proto: receive.proto,
                    local: receive.local,
                    to: receive.source.into(),
                    data: codec::encode_consult_response(res),
                });
                self.set_status(Status::Connecting);
                Ok(res)
            }
            Status::Connecting => {
                let request = codec::decode_connect_request(&mut receive.data)?;
                let connect = Connect {
                    proxy: self,
                    target: request.target,
                    source: receive.source,
                    local: receive.local,
                };
                Ok(Output::Connect(connect))
            }
            Status::Relaying { target } => {
                let transmit = Transmit {
                    proto: receive.proto,
                    local: receive.local,
                    to: target.clone(),
                    data: receive.data,
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

impl Default for Socks5Proxy {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Input {
    Receive(Receive),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Receive {
    pub proto: Protocol,
    pub local: SocketAddr,
    pub source: SocketAddr,
    pub data: Bytes,
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
    pub target: Address,
    pub source: SocketAddr,
    pub local: SocketAddr,
}

impl Connect<'_> {
    pub fn target(&self) -> &Address {
        &self.target
    }

    pub fn connected_success(self) -> Transmit {
        let res = ConnectResponse {
            status: ConnectedStatus::Succeeded,
            bind_address: self.local.into(),
        };
        let data = codec::encode_connect_response(res);
        self.proxy.set_status(Status::Relaying {
            target: self.target,
        });
        Transmit {
            proto: Protocol::Tcp,
            local: self.local,
            to: self.source.into(),
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
            local: self.local,
            to: self.source.into(),
            data,
        }
    }
}

#[cfg(test)]
mod tests;
