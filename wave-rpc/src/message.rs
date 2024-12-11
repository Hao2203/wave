use crate::{body::MessageBody, error::Error};
use bytes::Bytes;
use futures_lite::{stream::StreamExt, Stream};
use std::{future::Future, io};

#[cfg(feature = "bincode")]
pub mod bincode;
pub mod stream;

pub trait IoStream: Stream<Item = Result<Bytes, io::Error>> + Unpin + Send + 'static {}

impl<T> IoStream for T where T: Stream<Item = Result<Bytes, io::Error>> + Unpin + Send + 'static {}

pub trait FromBody<Ctx> {
    type Error: Into<Error>;

    fn from_body(
        ctx: &mut Ctx,
        body: impl IoStream,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;
}

pub trait IntoBody {
    fn into_body(self) -> impl MessageBody;
}

impl<Ctx: Send> FromBody<Ctx> for Bytes {
    type Error = Error;

    async fn from_body(_ctx: &mut Ctx, mut body: impl IoStream) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let data = body.next().await.transpose()?;

        Ok(data.unwrap_or_default())
    }
}
