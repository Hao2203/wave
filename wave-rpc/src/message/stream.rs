#![allow(unused)]
use super::{FromReader, SendTo};
use crate::transport::ConnectionReader;
use async_executor::Executor;
use async_trait::async_trait;
use derive_more::derive::Display;
use futures_lite::{
    stream::{self, Boxed},
    AsyncRead, AsyncWrite, StreamExt,
};
use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Stream<T> {
    reader: ConnectionReader,
    _marker: std::marker::PhantomData<T>,
}

impl<T> FromReader for Stream<T>
where
    T: Send + FromReader + 'static,
{
    type Error = Infallible;

    async fn from_reader(reader: ConnectionReader) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Stream {
            reader,
            _marker: std::marker::PhantomData,
        })
    }
}
