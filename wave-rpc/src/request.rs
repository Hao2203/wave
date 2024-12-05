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
use std::{
    convert::Infallible,
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

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }
}

// #[async_trait]
// impl<'a, T> FromReader<'a> for Request<T>
// where
//     T: FromReader<'a> + Send,
// {
//     type Error = std::io::Error;

//     async fn from_reader(
//         mut reader: impl AsyncRead + Send + Unpin + 'a,
//     ) -> Result<Self, Self::Error>
//     where
//         Self: Sized,
//     {
//         let header = Header::from_reader(&mut reader).await?;
//         let body = T::from_reader(reader)
//             .await
//             .map_err(|e| std::io::ErrorKind::InvalidData)?;
//         Ok(Self { header, body })
//     }
// }

// #[async_trait]
// impl<T> SendTo for Request<T>
// where
//     T: SendTo + Send,
// {
//     type Error = std::io::Error;

//     async fn send_to(
//         &mut self,
//         io: &mut (dyn futures::AsyncWrite + Send + Unpin),
//     ) -> std::result::Result<(), Self::Error> {
//         self.header.send_to(io).await?;
//         self.body.send_to(io).await.unwrap();
//         Ok(())
//     }
// }

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
}

pub(crate) struct HeaderCodec;

// #[async_trait]
// impl FromReader<'_> for Header {
//     type Error = std::io::Error;

//     async fn from_reader(mut reader: impl AsyncRead + Send + Unpin) -> Result<Self, Self::Error> {
//         let mut header_buf = Header::BUFFER;

//         reader.read_exact(&mut header_buf).await?;

//         let header: Header = Header::try_read_from_bytes(&header_buf)
//             .map_err(|e| std::io::ErrorKind::InvalidData)?;

//         Ok(header)
//     }
// }

// #[async_trait]
// impl SendTo for Header {
//     type Error = std::io::Error;

//     async fn send_to(
//         &mut self,
//         io: &mut (dyn futures::AsyncWrite + Send + Unpin),
//     ) -> std::result::Result<(), Self::Error> {
//         let header_bytes = self.as_bytes();
//         io.write_all(header_bytes).await
//     }
// }
