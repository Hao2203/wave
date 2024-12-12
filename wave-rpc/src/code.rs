#![allow(unused)]
use std::io;

use crate::message::FromStream;
use async_trait::async_trait;
use derive_more::derive::Display;
use futures_lite::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
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

    pub(crate) async fn from_reader(reader: &mut (impl AsyncRead + Unpin)) -> crate::Result<Self> {
        let mut buf = Self::BUFFER;
        reader.read_exact(&mut buf).await?;
        Ok(Code::try_read_from_bytes(&buf)?)
    }

    pub(crate) async fn write_into(
        self,
        writer: &mut (impl AsyncWrite + Unpin),
    ) -> crate::Result<(), io::Error> {
        writer.write_all(self.as_bytes()).await?;
        Ok(())
    }
}
