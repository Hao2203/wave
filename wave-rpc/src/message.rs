#![allow(unused)]
use async_stream::stream;
use async_trait::async_trait;
use bytes::Bytes;
use futures::{
    future::BoxFuture,
    pin_mut, ready,
    stream::{self, BoxStream},
    AsyncRead, AsyncWrite, StreamExt,
};
use std::{
    convert::Infallible,
    future::Future,
    pin::{pin, Pin},
    task::{Context, Poll},
};

pub trait Message<'a> {
    type Error: core::error::Error + Send;

    fn from_reader(
        io: impl AsyncRead + Send + Unpin + 'a,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized,
        Result<Self, Self::Error>: 'a;

    async fn write_in(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error>;
}

pub struct Stream<T> {
    reader: Box<dyn AsyncRead + Send + Unpin>,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T, E> Message<'static> for Stream<T>
where
    T: Send + for<'b> Message<'b, Error = E>,
    E: core::error::Error + Send,
{
    type Error = E;

    async fn from_reader(
        mut io: impl AsyncRead + Send + Unpin + 'static,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Stream {
            reader: Box::new(io),
            _marker: std::marker::PhantomData,
        })
    }

    async fn write_in(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error> {
        while let Some(item) = self.next().await {
            item?.write_in(io).await?
        }
        Ok(())
    }
}

impl<T, E> stream::Stream for Stream<T>
where
    T: Send + for<'b> Message<'b, Error = E>,
{
    type Item = Result<T, E>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let io = &mut self.get_mut().reader;
        let fut = async move {
            let item = T::from_reader(io).await;
            Some(item)
        };
        pin_mut!(fut);
        fut.poll(cx)
    }
}
