#![allow(unused)]
use crate::{
    body::Body,
    error::{Error, Result},
    message::FromBody,
    service::Version,
    ServiceDef,
};
use async_trait::async_trait;
use bytes::{Buf, BytesMut};
use futures_lite::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _};
use std::{
    convert::Infallible,
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio_util::{
    codec::{Decoder, Encoder, Framed, FramedRead},
    compat::FuturesAsyncReadCompatExt,
};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

pub struct Request {
    pub header: Header,
    pub body: Body,
}

impl Request {
    pub fn new(header: Header, body: Body) -> Self {
        Self { header, body }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn service_id(&self) -> u32 {
        self.header.service_id
    }

    pub fn service_version(&self) -> Version {
        Version::from(self.header.service_version)
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }
}

#[derive(Debug, Clone, Copy, TryFromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C, packed)]
pub struct Header {
    pub service_id: u32,
    pub service_version: u32,
}

impl Header {
    pub const SIZE: usize = 8;
    const BUFFER: [u8; Self::SIZE] = [0u8; Self::SIZE];

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        <Self as IntoBytes>::as_bytes(self)
    }

    pub(crate) async fn from_reader(reader: &mut (impl AsyncRead + Unpin)) -> Result<Self> {
        let mut buf = Self::BUFFER;
        reader.read_exact(&mut buf).await?;
        Ok(Header::try_read_from_bytes(&buf)?)
    }
}

pub(crate) struct HeaderCodec;

impl Decoder for HeaderCodec {
    type Item = Header;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.len() < Header::SIZE {
            return Ok(None);
        }
        let header = Header::try_read_from_bytes(&buf.split_to(Header::SIZE).freeze())
            .map_err(|e| io::Error::from(io::ErrorKind::InvalidData))?;
        Ok(Some(header))
    }
}

impl Encoder<Header> for HeaderCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Header, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.extend_from_slice(item.as_bytes());
        Ok(())
    }
}
