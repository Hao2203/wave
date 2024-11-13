use std::{
    pin::{pin, Pin},
    task::{Context, Poll},
};

use async_stream::stream;
use bytes::Bytes;
use futures::{
    future::BoxFuture,
    ready,
    stream::{self, BoxStream},
    StreamExt,
};
use tokio::io::AsyncRead;

pub trait Message<'a> {
    type Error: core::error::Error + Send;

    fn from_reader(
        io: impl AsyncRead + Send + Unpin + 'a,
    ) -> BoxFuture<'a, Result<Option<Self>, Self::Error>>
    where
        Self: Sized;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>>;

    fn to_stream(&'a mut self) -> MsgStream<'a, Self>
    where
        Self: Unpin,
    {
        MsgStream(self)
    }
}

pub struct Stream<'a, T> {
    inner: BoxStream<'a, T>,
}

impl<'a, T> Message<'a> for Stream<'a, T>
where
    T: Send + for<'b> Message<'b> + 'a,
{
    type Error = <T as Message<'a>>::Error;

    fn from_reader(
        mut io: impl AsyncRead + Send + Unpin + 'a,
    ) -> BoxFuture<'a, Result<Option<Self>, Self::Error>>
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

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        let poll = self.get_mut().inner.poll_next_unpin(cx);

        let res = ready!(poll).map(|item| {
            let item = pin!(item);
            item.poll_next(cx)
        });

        match res {
            Some(res) => res,
            None => Poll::Ready(None),
        }
    }
}

impl<'a, T> Message<'a> for Pin<Box<T>>
where
    T: Message<'a>,
{
    type Error = T::Error;
    fn from_reader(
        io: impl AsyncRead + Send + Unpin + 'a,
    ) -> BoxFuture<'a, Result<Option<Self>, Self::Error>>
    where
        Self: Sized,
    {
        let fut = async move {
            let item = T::from_reader(io).await?;
            Ok(item.map(Box::pin))
        };
        Box::pin(fut)
    }

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        self.get_mut().as_mut().poll_next(cx)
    }
}

pub(crate) struct MsgStream<'a, T: ?Sized>(&'a mut T);

impl<'a, T> stream::Stream for MsgStream<'a, T>
where
    T: Message<'a> + Unpin,
{
    type Item = Result<Bytes, T::Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let item = &mut self.get_mut().0;
        <T as Message>::poll_next(Pin::new(item), cx)
    }
}
