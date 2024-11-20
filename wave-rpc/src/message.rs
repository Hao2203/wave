use async_trait::async_trait;
use derive_more::derive::Display;
use futures::{io::AsyncReadExt, AsyncRead, AsyncWrite, AsyncWriteExt};
use std::convert::Infallible;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio_util::compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt};

use crate::{
    code::Code,
    error::{Error, RpcError},
};

pub mod stream;

#[async_trait]
pub trait FromReader<'a> {
    type Error: core::error::Error + Send;

    async fn from_reader(reader: impl AsyncRead + Send + Unpin + 'a) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

#[async_trait]
pub trait SendTo {
    type Error: core::error::Error + Send;

    async fn send_to(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error>;
}

#[async_trait]
impl<'a> FromReader<'a> for Box<dyn AsyncRead + Send + Unpin + 'a> {
    type Error = Infallible;

    async fn from_reader(reader: impl AsyncRead + Send + Unpin + 'a) -> Result<Self, Self::Error> {
        Ok(Box::new(reader))
    }
}

#[async_trait]
impl FromReader<'_> for String {
    type Error = std::io::Error;

    async fn from_reader(mut reader: impl AsyncRead + Send + Unpin) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        String::from_utf8(buf).map_err(|_| std::io::ErrorKind::InvalidData.into())
    }
}

#[async_trait]
impl SendTo for String {
    type Error = std::io::Error;

    async fn send_to(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error> {
        io.write_all(self.as_bytes()).await?;
        Ok(())
    }
}

#[derive(Debug, Display, derive_more::Error)]
pub enum ResultMessageError {
    DecodeTagFailed,
}

impl RpcError for ResultMessageError {
    fn code(&self) -> Code {
        match self {
            ResultMessageError::DecodeTagFailed => Code::InvalidMessage,
        }
    }
}

#[async_trait]
impl<T, E> FromReader<'_> for Result<T, E>
where
    for<'a> T: FromReader<'a, Error: Into<Error>>,
    for<'a> E: FromReader<'a, Error: Into<Error>> + std::marker::Send + derive_more::Error,
{
    type Error = Error;

    async fn from_reader(mut reader: impl AsyncRead + Send + Unpin) -> Result<Self, Self::Error> {
        let tag = { (&mut reader).compat().read_u8().await? };
        match tag {
            0 => Ok(Ok(T::from_reader(reader).await.map_err(Into::into)?)),
            1 => Ok(Err(E::from_reader(reader).await.map_err(Into::into)?)),
            _ => Err(ResultMessageError::DecodeTagFailed.into()),
        }
    }
}

#[async_trait]
impl<T, E> SendTo for Result<T, E>
where
    for<'a> T: SendTo<Error: Into<Error>> + Send,
    for<'a> E: SendTo<Error: Into<Error>> + Send + Send + core::error::Error,
{
    type Error = crate::error::Error;

    async fn send_to(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error> {
        match self {
            Ok(t) => {
                io.compat_write().write_u8(0).await?;
                t.send_to(io).await.map_err(Into::into)?;
            }
            Err(e) => {
                io.compat_write().write_u8(1).await?;
                e.send_to(io).await.map_err(Into::into)?;
            }
        }
        Ok(())
    }
}
