use super::{FromBody, IntoBody};
use crate::{body::MessageBody, error::BoxError};
use derive_more::derive::From;
use futures_lite::{
    stream::{self, Boxed},
    StreamExt,
};
use std::{convert::Infallible, sync::Arc};

pub enum Stream<T> {
    Body(Boxed<Result<Arc<[u8]>, Error>>),
    Stream(Boxed<T>),
}

#[derive(Debug, From)]
pub struct Error(pub BoxError);

impl From<Error> for BoxError {
    fn from(value: Error) -> Self {
        value.0
    }
}

impl<T, Ctx> FromBody for Stream<T>
where
    T: FromBody<Ctx = Ctx> + Send + Sized + 'static,
    Ctx: Send,
{
    type Ctx = Ctx;
    type Error = Infallible;

    async fn from_body(_ctx: &mut Self::Ctx, body: impl MessageBody) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self::Body(
            body.map(|data| data.map_err(Into::into).map_err(Into::into))
                .boxed(),
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
                .map(|item| item.map_err(Into::into).map_err(Into::into))
                .boxed(),
        }
    }
}

impl<T> Stream<T>
where
    T: FromBody<Ctx: Send> + Send + Sized + 'static,
{
    pub fn make_stream(
        self,
        ctx: &mut T::Ctx,
    ) -> impl futures_lite::Stream<Item = Result<T, T::Error>> + Send + Unpin + use<'_, T> {
        match self {
            Stream::Body(body) => stream::unfold((ctx, body), |(ctx, mut body)| async {
                if let Some(data) = body.next().await {
                    let item = T::from_body(ctx, stream::once(data)).await;
                    Some((item, (ctx, body)))
                } else {
                    None
                }
            })
            .boxed(),
            //  as BoxStream<'_, Result<T, T::Error>>,
            Stream::Stream(stream) => stream.map(Ok).boxed(),
        }
    }
}
