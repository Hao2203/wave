#![allow(unused)]
use super::{FromBody, MessageBody};
use crate::{
    body::{self, Body},
    transport::ConnectionReader,
};
use async_executor::Executor;
use async_stream::stream;
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

pub enum Stream<T> {
    Body(Body),
    Stream(Boxed<T>),
}

impl<T> FromBody for Stream<T> {
    type Error = Infallible;

    async fn from_body(body: Body) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self::Body(body))
    }
}

impl<T, E> futures_lite::Stream for Stream<T>
where
    T: Send + FromBody<Error = E>,
{
    type Item = Result<T, E>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            Self::Body(body) => {
                let data = ready!(body.poll_next(cx));
                if let Some(item) = data {
                    let item = pin!(T::from_body(body::Body::once(item)));
                    item.poll(cx).map(Some)
                } else {
                    Poll::Ready(None)
                }
            }
            Self::Stream(stream) => {
                let data = ready!(stream.poll_next(cx));
                Poll::Ready(data.map(Ok))
            }
        }
    }
}
