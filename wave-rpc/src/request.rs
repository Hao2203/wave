#![allow(unused)]
use crate::{
    error::{Error, Result},
    message::{FromReader, WriteIn},
    service::Version,
    ServiceDef,
};
use async_trait::async_trait;
use bytes::{Buf, BytesMut};
use futures::{future::BoxFuture, AsyncRead, AsyncReadExt, AsyncWriteExt, StreamExt};
use tokio_util::{
    codec::{Decoder, Encoder, Framed, FramedRead},
    compat::FuturesAsyncReadCompatExt,
};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

pub struct Request<T> {
    pub header: Header,
    pub body: T,
}

impl<T> Request<T> {
    // pub fn new<S>(req: S::Request<'a>, service_version: impl Into<Version>) -> Result<Self>
    // where
    //     S: ServiceDef,
    //     S::Request<'a>: FromReader<'a>,
    // {
    //     let header = Header {
    //         service_id: S::ID,
    //         service_version: service_version.into().into(),
    //     };
    //     todo!()
    // }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn service_id(&self) -> u32 {
        self.header.service_id
    }

    pub fn service_version(&self) -> Version {
        Version::from(self.header.service_version)
    }
}

impl<T> WriteIn for Request<T>
where
    T: WriteIn + Send,
{
    type Error = std::io::Error;

    fn write_in<'a>(
        &'a mut self,
        io: &'a mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> BoxFuture<'a, std::result::Result<(), Self::Error>> {
        let fut = async move {
            self.header.write_in(io).await?;
            self.body.write_in(io).await.unwrap();
            Ok(())
        };
        Box::pin(fut)
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

    #[inline]
    pub fn buffer() -> [u8; Self::SIZE] {
        [0u8; Self::SIZE]
    }

    pub async fn from_reader(reader: &mut (impl AsyncRead + Unpin)) -> Result<Self> {
        let mut header_buf = Header::buffer();

        let _ = reader.read(&mut header_buf).await.unwrap();

        let header: Header = Header::try_read_from_bytes(&header_buf[..])
            .map_err(|e| anyhow::anyhow!("Can't parse header from bytes, error: {}", e))
            .unwrap();

        Ok(header)
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        <Self as IntoBytes>::as_bytes(self)
    }
}

pub(crate) struct HeaderCodec;

impl Decoder for HeaderCodec {
    type Item = Header;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < Header::SIZE {
            return Ok(None);
        }

        let header = {
            let mut header_buf = Header::buffer();
            src.copy_to_slice(&mut header_buf[..]);
            *Header::try_ref_from_bytes(&header_buf[..]).unwrap()
        };

        Ok(Some(header))
    }
}

impl FromReader<'_> for Header {
    type Error = std::io::Error;

    async fn from_reader(reader: impl AsyncRead + Unpin) -> Result<Self, Self::Error> {
        let mut framed = FramedRead::new(reader.compat(), HeaderCodec);
        framed
            .next()
            .await
            .ok_or(std::io::ErrorKind::UnexpectedEof)?
    }
}

impl WriteIn for Header {
    type Error = std::io::Error;

    fn write_in<'a>(
        &'a mut self,
        io: &'a mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> futures::future::BoxFuture<'a, std::result::Result<(), Self::Error>> {
        Box::pin(async move {
            let header_bytes = self.as_bytes();
            io.write_all(header_bytes).await
        })
    }
}
