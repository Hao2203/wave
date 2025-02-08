use super::Error;
use crate::Address;
use bytes::{Buf, BufMut, Bytes, BytesMut};
pub use consts::*;
use derive_more::derive::Display;
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    sync::Arc,
};

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
pub enum AuthMethod {
    None = SOCKS5_AUTH_METHOD_NONE,
    Gssapi = SOCKS5_AUTH_METHOD_GSSAPI,
    Password = SOCKS5_AUTH_METHOD_PASSWORD,
    NotAcceptable = SOCKS5_AUTH_METHOD_NOT_ACCEPTABLE,
}

impl TryFrom<u8> for AuthMethod {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            SOCKS5_AUTH_METHOD_NONE => Ok(AuthMethod::None),
            SOCKS5_AUTH_METHOD_GSSAPI => Ok(AuthMethod::Gssapi),
            SOCKS5_AUTH_METHOD_PASSWORD => Ok(AuthMethod::Password),
            SOCKS5_AUTH_METHOD_NOT_ACCEPTABLE => Ok(AuthMethod::NotAcceptable),
            _ => Err(Error::InvalidMethod { method: value }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakeRequest {
    pub n_methods: u8,
    pub methods: Arc<[AuthMethod]>,
}

impl HandshakeRequest {
    /// |VER | NMETHODS | METHODS  |
    /// |:--:|:--------:|:-------:|
    /// | 1  |    1     | 1 to 255 |
    pub fn decode(buf: &mut BytesMut) -> Result<Option<Self>, Error> {
        if buf.remaining() < 2 {
            return Ok(None);
        }
        let version = buf.get_u8();
        if version != 5 {
            return Err(Error::InvalidVersion { version });
        }
        let n_methods = buf.get_u8();
        let methods = buf.split_to(n_methods as usize);
        let methods = methods
            .into_iter()
            .map(AuthMethod::try_from)
            .collect::<Result<Arc<[_]>, Error>>()?;

        Ok(Some(HandshakeRequest { n_methods, methods }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandshakeResponse(pub AuthMethod);

impl HandshakeResponse {
    /// +----+--------+
    /// |VER | METHOD |
    /// +----+--------+
    /// | 1  |   1    |
    /// +----+--------+
    pub fn encode(self) -> Bytes {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_u8(5);
        buf.put_u8(self.0 as u8);
        buf.freeze()
    }

    pub fn is_acceptable(&self) -> bool {
        self.0 != AuthMethod::NotAcceptable
    }
}

pub struct ConnectRequest {
    pub command: Command,
    pub target: Address,
}

impl ConnectRequest {
    /// |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
    /// |:--:|:---:|:-----:|:----:|:--------:|:--------:|
    /// | 1  |  1  | X'00' |  1   | Variable |    2     |
    pub fn decode(buf: &mut BytesMut) -> Result<Option<Self>, Error> {
        if buf.remaining() < 4 {
            return Ok(None);
        }
        let version = buf.get_u8();
        if version != 5 {
            return Err(Error::InvalidVersion { version });
        }
        let command = buf.get_u8().try_into()?;
        let _reserved = buf.get_u8();
        let (_addr_type, target) = if let Some(addr) = decode_address(buf)? {
            addr
        } else {
            return Ok(None);
        };
        Ok(Some(ConnectRequest { command, target }))
    }
}

pub struct ConnectResponse {
    pub status: ConnectedStatus,
    pub bind_address: Address,
}

impl ConnectResponse {
    /// |VER|REP|RSV|ATYP|BND.ADDR|BND.PORT|
    /// |---|---|---|---|---|---|
    /// |1|1| '00'|1 |Variable|2|
    pub fn encode(self) -> Bytes {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_u8(5);
        buf.put_u8(self.status as u8);
        buf.put_u8(0);
        buf.put(encode_address(self.bind_address));
        buf.freeze()
    }
}

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq)]
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
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            SOCKS5_CMD_TCP_CONNECT => Ok(Command::Connect),
            SOCKS5_CMD_TCP_BIND => Ok(Command::Bind),
            SOCKS5_CMD_UDP_ASSOCIATE => Ok(Command::UdpAssociate),
            _ => Err(Error::InvalidCommand { command: value }),
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
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            SOCKS5_ADDR_TYPE_IPV4 => Ok(AddrType::V4),
            SOCKS5_ADDR_TYPE_IPV6 => Ok(AddrType::V6),
            SOCKS5_ADDR_TYPE_DOMAIN_NAME => Ok(AddrType::Domain),
            _ => Err(Error::InvalidAddrType { addr_type: value }),
        }
    }
}

pub fn decode_address(mut buf: impl Buf) -> Result<Option<(AddrType, Address)>, Error> {
    let addr_type = buf.get_u8().try_into()?;
    let address = match addr_type {
        AddrType::V4 => {
            if buf.remaining() < 8 {
                return Ok(None);
            }
            Address::Ip(SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(buf.get_u8(), buf.get_u8(), buf.get_u8(), buf.get_u8()),
                buf.get_u16(),
            )))
        }
        AddrType::V6 => {
            if buf.remaining() < 18 {
                return Ok(None);
            }
            Address::Ip(SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::new(
                    buf.get_u16(),
                    buf.get_u16(),
                    buf.get_u16(),
                    buf.get_u16(),
                    buf.get_u16(),
                    buf.get_u16(),
                    buf.get_u16(),
                    buf.get_u16(),
                ),
                buf.get_u16(),
                0,
                0,
            )))
        }
        AddrType::Domain => {
            let len = buf.get_u8();
            if buf.remaining() < len as usize + 2 {
                return Ok(None);
            }
            let domain = buf.copy_to_bytes(len as usize);
            let domain = String::from_utf8(domain.into())?;
            let port = buf.get_u16();
            let address = format!("{}:{}", domain, port);
            address.parse()?
        }
    };
    Ok(Some((addr_type, address)))
}

pub fn encode_address(address: Address) -> Bytes {
    match address {
        Address::Ip(addr) => {
            let mut buf = BytesMut::with_capacity(18);
            match addr {
                SocketAddr::V4(addr) => {
                    buf.put_u8(AddrType::V4 as u8);
                    addr.ip().octets().into_iter().for_each(|x| buf.put_u8(x));
                    buf.put_u16(addr.port());
                }
                SocketAddr::V6(addr) => {
                    buf.put_u8(AddrType::V6 as u8);
                    addr.ip()
                        .segments()
                        .into_iter()
                        .for_each(|x| buf.put_u16(x));
                    buf.put_u16(addr.port());
                }
            }
            buf.freeze()
        }
        Address::Domain(domain, port) => {
            let mut buf = BytesMut::with_capacity(2 + domain.len());
            buf.put_u8(AddrType::Domain as u8);
            buf.put_u8(domain.len() as u8);
            buf.put(domain.as_bytes());
            buf.put_u16(port);
            buf.freeze()
        }
    }
}
