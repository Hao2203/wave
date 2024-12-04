#![allow(unused)]
use crate::message::{FromReader, SendTo};
use async_trait::async_trait;
use derive_more::derive::Display;
use futures_lite::AsyncWrite;
use zerocopy::{Immutable, IntoBytes, TryFromBytes};

#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    Eq,
    PartialEq,
    zerocopy::TryFromBytes,
    zerocopy::IntoBytes,
    Immutable,
)]
#[non_exhaustive]
#[repr(u32)]
pub enum Code {
    Ok,
    IoError,
    InvalidMessage,
    InternalServerError,
}

impl Code {
    const SIZE: usize = 4;
    const BUFFER: [u8; Self::SIZE] = [0u8; Self::SIZE];
}

// #[async_trait::async_trait]
// impl FromReader<'_> for Code {
//     type Error = std::io::Error;

//     async fn from_reader(
//         mut reader: impl futures::AsyncRead + Send + Unpin,
//     ) -> Result<Self, Self::Error> {
//         let mut buf = Code::BUFFER;
//         reader.read_exact(&mut buf).await?;
//         let code = Code::try_read_from_bytes(&buf).map_err(|_| std::io::ErrorKind::InvalidData)?;
//         Ok(code)
//     }
// }

// #[async_trait]
// impl SendTo for Code {
//     type Error = std::io::Error;
//     async fn send_to(
//         &mut self,
//         io: &mut (dyn AsyncWrite + Send + Unpin),
//     ) -> Result<(), Self::Error> {
//         io.write_all(self.as_bytes()).await?;
//         Ok(())
//     }
// }
