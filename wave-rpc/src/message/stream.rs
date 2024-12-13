#![allow(unused)]
use super::{BytesStream, FromStream, IntoStream, TryBytesStream};
use crate::{body::MessageBody, error::Error};
use bytes::Bytes;
use futures_lite::{
    stream::{self, Boxed},
    StreamExt,
};
use std::{convert::Infallible, io};

pub enum Stream<T> {
    Body(Boxed<Bytes>),
    Stream(Boxed<T>),
}

impl<T> FromStream for Stream<T> {
    type Error = Infallible;

    async fn from_stream(body: impl BytesStream) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self::Body(body.boxed()))
    }
}

impl<T> IntoStream for Stream<T>
where
    T: IntoStream + 'static,
{
    type Error = T::Error;

    fn into_stream(self) -> impl TryBytesStream<Error = Self::Error> {
        match self {
            Stream::Body(body) => body.map(Ok).boxed(),
            Stream::Stream(stream) => stream.map(|data| data.into_stream()).flatten().boxed(),
        }
    }
}

impl<T> Stream<T> {
    pub fn make_stream(self) -> impl futures_lite::Stream<Item = Result<T, T::Error>> + Send + Unpin
    where
        T: FromStream + 'static,
    {
        match self {
            Stream::Body(body) => stream::unfold(body, |mut body| async {
                if let Some(data) = body.next().await {
                    let item = T::from_stream(stream::once(data)).await;
                    Some((item, body))
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
