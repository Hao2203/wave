#![allow(unused)]
use async_trait::async_trait;
use derive_more::derive::Display;
use futures_lite::{AsyncRead, AsyncWrite};
use std::{convert::Infallible, future::Future};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio_util::compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt};

use crate::{
    code::Code,
    error::{Error, RpcError},
    transport::ConnectionReader,
};

pub mod stream;

pub trait FromReader {
    type Error: core::error::Error + Send;

    fn from_reader(
        reader: &ConnectionReader,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send
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

// #[async_trait]
// impl SendTo for String {
//     type Error = std::io::Error;

//     async fn send_to(
//         &mut self,
//         io: &mut (dyn AsyncWrite + Send + Unpin),
//     ) -> Result<(), Self::Error> {
//         io.write_all(self.as_bytes()).await?;
//         Ok(())
//     }
// }

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
