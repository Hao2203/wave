#![allow(unused)]
use async_trait::async_trait;
use bytes::Bytes;
use derive_more::derive::Display;
use futures_lite::{AsyncRead, AsyncWrite, Stream};
use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio_util::compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt};

use crate::{
    body::Body,
    code::Code,
    error::{Error, RpcError},
    transport::ConnectionReader,
};

pub mod stream;

pub trait FromBody {
    type Error: core::error::Error + Send;

    fn from_body(body: Body) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;
}

pub trait MessageBody: Stream<Item = Result<Self::Chunk, Self::Error>> {
    type Error: core::error::Error + Send;
    type Chunk: AsRef<[u8]> + 'static;
}

impl<S, T, E> MessageBody for S
where
    S: Stream<Item = Result<T, E>>,
    T: AsRef<[u8]> + 'static,
    E: core::error::Error + Send,
{
    type Error = E;
    type Chunk = T;
}

pub trait BodyStream {
    type Error: core::error::Error + Send;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Bytes, Self::Error>>>;
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
