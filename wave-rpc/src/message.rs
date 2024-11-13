use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::error::Error;
use async_stream::stream;
use bytes::Bytes;
use futures::{future::BoxFuture, stream::BoxStream, StreamExt};
use tokio::io::AsyncRead;

pub trait Message<'a> {
    fn from_reader(
        io: impl AsyncRead + Send + Unpin + 'a,
    ) -> BoxFuture<'a, Result<Option<Self>, Error>>
    where
        Self: Sized;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>>;
}

pub struct Stream<'a, T> {
    inner: BoxStream<'a, T>,
}

impl<'a, T> Message<'a> for Stream<'a, T>
where
    T: Send + for<'b> Message<'b> + 'a,
{
    fn from_reader(
        mut io: impl AsyncRead + Send + Unpin + 'a,
    ) -> BoxFuture<'a, Result<Option<Self>, Error>>
    where
        Self: Sized,
    {
        let stream = stream! {
            while let Ok(Some(item)) = T::from_reader(&mut io).await {
                yield item
            }
        };
        Box::pin(async move {
            Ok(Some(Stream {
                inner: stream.boxed(),
            }))
        })
    }

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>> {
        self.get_mut().inner.map(Ok).poll_next_unpin(cx)
    }
}
