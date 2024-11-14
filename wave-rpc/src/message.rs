#![allow(unused)]
use async_stream::stream;
use async_trait::async_trait;
use bytes::Bytes;
use futures::{
    future::BoxFuture,
    ready,
    stream::{self, BoxStream},
    AsyncRead, AsyncWrite, StreamExt,
};
use std::{
    pin::{pin, Pin},
    task::{Context, Poll},
};

#[async_trait]
pub trait Message<'a> {
    type Error: core::error::Error + Send;

    async fn from_reader(
        io: impl AsyncRead + Send + Unpin + 'a,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized;

    async fn write_in(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error>;
}

pub struct Stream<'a, T> {
    inner: BoxStream<'a, T>,
}

#[async_trait]
impl<'a, T> Message<'a> for Stream<'a, T>
where
    T: Send + for<'b> Message<'b> + 'a,
{
    type Error = <T as Message<'a>>::Error;

    async fn from_reader(
        mut io: impl AsyncRead + Send + Unpin + 'a,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized,
    {
        let stream = stream! {
            while let Some(Ok(item)) = T::from_reader(&mut io).await.transpose() {
                yield item
            }
        };

        Ok(Some(Stream {
            inner: stream.boxed(),
        }))
    }

    async fn write_in(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error> {
        while let Some(mut item) = self.inner.next().await {
            item.write_in(io).await?
        }
        Ok(())
    }
}
