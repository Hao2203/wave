#![allow(unused)]
use super::{FromBody, IntoBody, IoStream};
use crate::{body::MessageBody, error::Error};
use bytes::Bytes;
use futures_lite::{
    stream::{self, Boxed},
    StreamExt,
};
use std::{convert::Infallible, io};

pub enum Stream<T> {
    Body(Boxed<Result<Bytes, io::Error>>),
    Stream(Boxed<T>),
}

impl<T, Ctx> FromBody<Ctx> for Stream<T>
where
    Ctx: Send,
{
    type Error = Infallible;

    async fn from_body(_ctx: &mut Ctx, body: impl IoStream) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self::Body(body.boxed()))
    }
}

// impl<T> IntoBody for Stream<T>
// where
//     T: IntoBody + 'static,
// {
//     fn into_body(self) -> impl MessageBody {
//         match self {
//             Stream::Body(body) => body,
//             Stream::Stream(stream) => stream
//                 .map(|item| item.into_body())
//                 .flatten()
//                 .map(|item| item.map_err(Into::into))
//                 .boxed(),
//         }
//     }
// }

impl<T> Stream<T> {
    pub fn make_stream<Ctx>(
        self,
        ctx: &mut Ctx,
    ) -> impl futures_lite::Stream<Item = Result<T, T::Error>> + Send + Unpin + use<'_, Ctx, T>
    where
        T: FromBody<Ctx> + 'static,
        Ctx: Send,
    {
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
