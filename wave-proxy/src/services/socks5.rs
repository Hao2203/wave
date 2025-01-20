#![allow(unused_imports)]
use super::*;
use crate::{
    error::WithKind, Address, Connection, Error, ErrorKind, Incoming, ProxyApp, ProxyService,
    ProxyStatus, Result,
};
use bytes::{Buf, BufMut, Bytes, BytesMut};
pub use consts::*;
use std::{
    collections::VecDeque,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs},
    sync::Arc,
    time::Duration,
};

pub mod codec;

#[rustfmt::skip]
pub mod consts {
    pub const SOCKS5_VERSION:                          u8 = 0x05;

    pub const SOCKS5_AUTH_METHOD_NONE:                 u8 = 0x00;
    pub const SOCKS5_AUTH_METHOD_GSSAPI:               u8 = 0x01;
    pub const SOCKS5_AUTH_METHOD_PASSWORD:             u8 = 0x02;
    pub const SOCKS5_AUTH_METHOD_NOT_ACCEPTABLE:       u8 = 0xff;

    pub const SOCKS5_CMD_TCP_CONNECT:                  u8 = 0x01;
    pub const SOCKS5_CMD_TCP_BIND:                     u8 = 0x02;
    pub const SOCKS5_CMD_UDP_ASSOCIATE:                u8 = 0x03;

    pub const SOCKS5_ADDR_TYPE_IPV4:                   u8 = 0x01;
    pub const SOCKS5_ADDR_TYPE_DOMAIN_NAME:            u8 = 0x03;
    pub const SOCKS5_ADDR_TYPE_IPV6:                   u8 = 0x04;

    pub const SOCKS5_REPLY_SUCCEEDED:                  u8 = 0x00;
    pub const SOCKS5_REPLY_GENERAL_FAILURE:            u8 = 0x01;
    pub const SOCKS5_REPLY_CONNECTION_NOT_ALLOWED:     u8 = 0x02;
    pub const SOCKS5_REPLY_NETWORK_UNREACHABLE:        u8 = 0x03;
    pub const SOCKS5_REPLY_HOST_UNREACHABLE:           u8 = 0x04;
    pub const SOCKS5_REPLY_CONNECTION_REFUSED:         u8 = 0x05;
    pub const SOCKS5_REPLY_TTL_EXPIRED:                u8 = 0x06;
    pub const SOCKS5_REPLY_COMMAND_NOT_SUPPORTED:      u8 = 0x07;
    pub const SOCKS5_REPLY_ADDRESS_TYPE_NOT_SUPPORTED: u8 = 0x08;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum Method {
    None = SOCKS5_AUTH_METHOD_NONE,
    Gssapi = SOCKS5_AUTH_METHOD_GSSAPI,
    Password = SOCKS5_AUTH_METHOD_PASSWORD,
    NotAcceptable = SOCKS5_AUTH_METHOD_NOT_ACCEPTABLE,
}

impl TryFrom<u8> for Method {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            SOCKS5_AUTH_METHOD_NONE => Ok(Method::None),
            SOCKS5_AUTH_METHOD_GSSAPI => Ok(Method::Gssapi),
            SOCKS5_AUTH_METHOD_PASSWORD => Ok(Method::Password),
            SOCKS5_AUTH_METHOD_NOT_ACCEPTABLE => Ok(Method::NotAcceptable),
            _ => Err(Error::message(
                ErrorKind::UnSupportedProxyProtocol,
                "Invalid socks5 method",
            )),
        }
    }
}

pub struct Socks5Proxy {
    recvs: VecDeque<Input>,
    status: Status,
}

pub enum Status {
    Consult,
    Connect,
    Reley { target: Address },
}

impl Socks5Proxy {
    pub fn poll_output(&mut self) -> Result<Output> {
        self.recvs
            .pop_front()
            .map(|input| match input {
                Input::Receive(receive) => self.process_receive(receive),
            })
            .unwrap_or(Ok(Output::Pending))
    }

    fn process_receive(&mut self, mut receive: Receive) -> Result<Output> {
        match &self.status {
            Status::Consult => {
                let request = codec::decode_consult_request(&mut receive.data)?;
                let res = self.process_consult_request(request)?;
                let res = Output::Transmit(Transmit {
                    proto: receive.proto,
                    local: receive.local,
                    to: receive.source.into(),
                    data: codec::encode_consult_response(res),
                });
                self.set_status(Status::Connect);
                Ok(res)
            }
            Status::Connect => {
                let request = codec::decode_connect_request(&mut receive.data)?;
                let connect = Connect {
                    proxy: self,
                    target: request.target,
                    source: receive.source,
                    local: receive.local,
                };
                Ok(Output::Connect(connect))
            }
            Status::Reley { target } => {
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
        if !request.methods.contains(&Method::None) {
            return Err(Error::message(
                ErrorKind::UnSupportedProxyProtocol,
                "Invalid socks5 request",
            ));
        }
        Ok(ConsultResponse(Method::None))
    }

    fn set_status(&mut self, status: Status) {
        self.status = status;
    }
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

pub enum Output<'a> {
    Pending,
    Transmit(Transmit),
    Connect(Connect<'a>),
}

pub struct Transmit {
    pub proto: Protocol,
    pub local: SocketAddr,
    pub to: Address,
    pub data: Bytes,
}

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
            address: self.target.clone(),
        };
        let data = codec::encode_connect_response(res);
        self.proxy.set_status(Status::Reley {
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
            address: self.target,
        };
        let data = codec::encode_connect_response(res);
        self.proxy.set_status(Status::Consult);
        Transmit {
            proto: Protocol::Tcp,
            local: self.local,
            to: self.source.into(),
            data,
        }
    }
}

/// |VER | NMETHODS | METHODS  |
/// |:--:|:--------:|:-------:|
/// | 1  |    1     | 1 to 255 |
pub struct ConsultRequest {
    pub n_methods: u8,
    pub methods: Vec<Method>,
}

/// +----+--------+
/// |VER | METHOD |
/// +----+--------+
/// | 1  |   1    |
/// +----+--------+
pub struct ConsultResponse(pub Method);

pub struct ConnectRequest {
    pub command: Command,
    pub target: Address,
}

pub struct ConnectResponse {
    pub status: ConnectedStatus,
    pub address: Address,
}

#[repr(u8)]
pub enum ConnectedStatus {
    Succeeded = SOCKS5_REPLY_SUCCEEDED,
    GeneralServerFailure = SOCKS5_REPLY_GENERAL_FAILURE,
    ConnectionNotAllowed = SOCKS5_REPLY_CONNECTION_NOT_ALLOWED,
    NetworkUnreachable = SOCKS5_REPLY_NETWORK_UNREACHABLE,
    HostUnreachable = SOCKS5_REPLY_HOST_UNREACHABLE,
    ConnectionRefused = SOCKS5_REPLY_CONNECTION_REFUSED,
    TtlExpired = SOCKS5_REPLY_TTL_EXPIRED,
    CommandNotSupported = SOCKS5_REPLY_COMMAND_NOT_SUPPORTED,
    AddressTypeNotSupported = SOCKS5_REPLY_ADDRESS_TYPE_NOT_SUPPORTED,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Command {
    Connect = SOCKS5_CMD_TCP_CONNECT,
    Bind = SOCKS5_CMD_TCP_BIND,
    UdpAssociate = SOCKS5_CMD_UDP_ASSOCIATE,
}

impl TryFrom<u8> for Command {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self> {
        match value {
            SOCKS5_CMD_TCP_CONNECT => Ok(Command::Connect),
            SOCKS5_CMD_TCP_BIND => Ok(Command::Bind),
            SOCKS5_CMD_UDP_ASSOCIATE => Ok(Command::UdpAssociate),
            _ => Err(Error::message(
                ErrorKind::UnSupportedProxyProtocol,
                "Invalid socks5 command",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AddrType {
    V4 = SOCKS5_ADDR_TYPE_IPV4,
    V6 = SOCKS5_ADDR_TYPE_IPV6,
    Domain = SOCKS5_ADDR_TYPE_DOMAIN_NAME,
}

impl TryFrom<u8> for AddrType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self> {
        match value {
            SOCKS5_ADDR_TYPE_IPV4 => Ok(AddrType::V4),
            SOCKS5_ADDR_TYPE_IPV6 => Ok(AddrType::V6),
            SOCKS5_ADDR_TYPE_DOMAIN_NAME => Ok(AddrType::Domain),
            _ => Err(Error::message(
                ErrorKind::UnSupportedProxyProtocol,
                "Invalid socks5 address type",
            )),
        }
    }
}
