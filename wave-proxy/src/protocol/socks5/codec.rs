use super::{Error, types::*};
use crate::Address;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    sync::Arc,
};

pub fn decode_consult_request(mut buf: impl Buf) -> Result<HandshakeRequest, Error> {
    if buf.remaining() < 2 {
        return Err(Error::LengthNotEnough {
            len: buf.remaining(),
        });
    }
    let version = buf.get_u8();
    if version != 5 {
        return Err(Error::InvalidVersion { version });
    }
    let n_methods = buf.get_u8();
    let methods = &buf.chunk()[0..n_methods as usize];
    let methods = methods
        .iter()
        .map(|x| (*x).try_into())
        .collect::<Result<Arc<[_]>, Error>>()?;

    Ok(HandshakeRequest { n_methods, methods })
}

pub fn encode_consult_response(response: HandshakeResponse) -> Bytes {
    let mut buf = BytesMut::with_capacity(2);
    buf.put_u8(5);
    buf.put_u8(response.0 as u8);
    buf.freeze()
}

/// |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
/// |:--:|:---:|:-----:|:----:|:--------:|:--------:|
/// | 1  |  1  | X'00' |  1   | Variable |    2     |
pub fn decode_connect_request(mut buf: impl Buf) -> Result<ConnectRequest, Error> {
    if buf.remaining() < 4 {
        return Err(Error::LengthNotEnough {
            len: buf.remaining(),
        });
    }
    let version = buf.get_u8();
    if version != 5 {
        return Err(Error::InvalidVersion { version });
    }
    let command = buf.get_u8().try_into()?;
    let _reserved = buf.get_u8();
    let (_addr_type, target) = decode_address(buf)?;
    Ok(ConnectRequest { command, target })
}

/// |VER|REP|RSV|ATYP|BND.ADDR|BND.PORT|
/// |---|---|---|---|---|---|
/// |1|1| '00'|1 |Variable|2|
pub fn encode_connect_response(response: ConnectResponse) -> Bytes {
    let mut buf = BytesMut::with_capacity(2);
    buf.put_u8(5);
    buf.put_u8(response.status as u8);
    buf.put_u8(0);
    buf.put(encode_address(response.bind_address));
    buf.freeze()
}

pub fn decode_address(mut buf: impl Buf) -> Result<(AddrType, Address), Error> {
    let addr_type = buf.get_u8().try_into()?;
    let address = match addr_type {
        AddrType::V4 => {
            if buf.remaining() < 8 {
                return Err(Error::LengthNotEnough {
                    len: buf.remaining(),
                });
            }
            Address::Ip(SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(buf.get_u8(), buf.get_u8(), buf.get_u8(), buf.get_u8()),
                buf.get_u16(),
            )))
        }
        AddrType::V6 => {
            if buf.remaining() < 18 {
                return Err(Error::LengthNotEnough {
                    len: buf.remaining(),
                });
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
                return Err(Error::LengthNotEnough {
                    len: buf.remaining(),
                });
            }
            let domain = buf.copy_to_bytes(len as usize);
            let domain = String::from_utf8(domain.into())?;
            let port = buf.get_u16();
            let address = format!("{}:{}", domain, port);
            address.parse()?
        }
    };
    Ok((addr_type, address))
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
