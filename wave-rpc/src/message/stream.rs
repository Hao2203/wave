use super::{FromBody, IntoBody};
use crate::{body::MessageBody, error::BoxError};
use futures_lite::{
    ready,
    stream::{self, Boxed},
    StreamExt,
};
use std::{
    convert::Infallible,
    future::Future,
    pin::{pin, Pin},
    sync::Arc,
    task::{Context, Poll},
};

pub enum Stream<T> {
    Body(Boxed<Result<Arc<[u8]>, BoxError>>),
    Stream(Boxed<T>),
}

impl<T> FromBody for Stream<T> {
    type Error = Infallible;

    async fn from_body(body: impl MessageBody) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self::Body(
            body.map(|data| data.map_err(Into::into)).boxed(),
        ))
    }
}

impl<T> IntoBody for Stream<T>
where
    T: IntoBody + 'static,
{
    fn into_body(self) -> impl MessageBody {
        match self {
            Stream::Body(body) => body,
            Stream::Stream(stream) => stream
                .map(|item| item.into_body())
                .flatten()
                .map(|item| item.map_err(Into::into))
                .boxed(),
        }
    }
}

impl<T, E> futures_lite::Stream for Stream<T>
where
    T: Send + FromBody<Error = E> + Sized,
{
    type Item = Result<T, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            Self::Body(body) => {
                let data = ready!(body.poll_next(cx));
                if let Some(item) = data {
                    let item = item.unwrap();
                    let item = pin!(T::from_body(stream::once(Ok::<_, Infallible>(item))));
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
