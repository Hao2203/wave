use super::Error;
use crate::Address;
pub use consts::*;
use derive_more::derive::Display;

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

/// |VER | NMETHODS | METHODS  |
/// |:--:|:--------:|:-------:|
/// | 1  |    1     | 1 to 255 |
pub struct ConsultRequest {
    pub n_methods: u8,
    pub methods: Vec<AuthMethod>,
}

/// +----+--------+
/// |VER | METHOD |
/// +----+--------+
/// | 1  |   1    |
/// +----+--------+
pub struct ConsultResponse(pub AuthMethod);

pub struct ConnectRequest {
    pub command: Command,
    pub target: Address,
}

pub struct ConnectResponse {
    pub status: ConnectedStatus,
    pub bind_address: Address,
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
