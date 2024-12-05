#![allow(unused)]
use super::{FromReader, SendTo};
use crate::transport::ConnectionReader;
use async_executor::Executor;
use async_trait::async_trait;
use derive_more::derive::Display;
use futures_lite::{
    ready,
    stream::{self, Boxed},
    AsyncRead, AsyncWrite, StreamExt,
};
use std::{
    convert::Infallible,
    future::Future,
    pin::{pin, Pin},
    task::{Context, Poll},
};

pub struct Stream<T> {
    reader: ConnectionReader,
    _marker: std::marker::PhantomData<T>,
}

impl<T> FromReader for Stream<T> {
    type Error = Infallible;

    async fn from_reader(reader: &ConnectionReader) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Stream {
            reader: reader.clone(),
            _marker: std::marker::PhantomData,
        })
    }
}

impl<T, E> futures_lite::Stream for Stream<T>
where
    T: Send + FromReader<Error = E> + 'static,
{
    type Item = Result<T, E>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let reader = &self.reader;
        let item = T::from_reader(reader);
        let item = pin!(item);
        item.poll(cx).map(Some)
    }
}
