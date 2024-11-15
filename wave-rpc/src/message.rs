use std::{
    future::Future,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use futures::{future::BoxFuture, AsyncRead, AsyncWrite};
use pin_project::pin_project;

use crate::error;

pub mod stream;

pub trait Message<'a> {
    type Error: core::error::Error + Send;

    fn from_reader(
        reader: impl AsyncRead + Send + Unpin + 'a,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;

    fn write_in(
        &'a mut self,
        io: &'a mut (dyn AsyncWrite + Send + Unpin),
    ) -> BoxFuture<'a, Result<(), Self::Error>>;

    fn into_boxed(self) -> Box<dyn Message<'a, Error = error::Error> + Send + 'a>
    where
        Self: Sized + Send + 'a,
        Self::Error: Into<error::Error>,
    {
        Box::new(BoxMessage(self))
    }
}

pub struct BoxMessage<T>(pub T);

impl<'a, T, E> Message<'a> for BoxMessage<T>
where
    T: Message<'a, Error = E> + Send,
    E: Into<error::Error> + core::error::Error + Send,
{
    type Error = error::Error;

    fn from_reader(
        reader: impl AsyncRead + Send + Unpin + 'a,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized,
    {
        BoxMessageFut(T::from_reader(reader))
    }

    fn write_in(
        &'a mut self,
        io: &'a mut (dyn AsyncWrite + Send + Unpin),
    ) -> BoxFuture<'a, Result<(), Self::Error>> {
        Box::pin(async move { self.0.write_in(io).await.map_err(Into::into) })
    }
}

#[pin_project]
pub struct BoxMessageFut<T>(#[pin] pub T);

impl<'a, Fut, T, E> Future for BoxMessageFut<Fut>
where
    Fut: Future<Output = Result<T, E>>,
    E: Into<error::Error>,
{
    type Output = Result<BoxMessage<T>, error::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        let res = this.0.poll(cx).map_err(Into::into);
        res.map(|r| r.map(BoxMessage))
    }
}
