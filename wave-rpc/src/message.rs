use crate::body::MessageBody;
use bytes::Bytes;
use futures_lite::{stream::StreamExt, Stream};
use std::{error::Error, future::Future, io};

#[cfg(feature = "bincode")]
pub mod bincode;
pub mod stream;

pub trait IoBytesStream: Stream<Item = Result<Bytes, io::Error>> + Unpin + Send + 'static {}

impl<T> IoBytesStream for T where T: Stream<Item = Result<Bytes, io::Error>> + Unpin + Send + 'static
{}

pub trait FromBody<Ctx> {
    type Error: Error;

    fn from_body(
        ctx: &mut Ctx,
        body: impl IoBytesStream,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;
}

pub trait IntoBody {
    fn into_body(self) -> impl MessageBody;
}

impl<Ctx: Send> FromBody<Ctx> for Bytes {
    type Error = io::Error;

    async fn from_body(_ctx: &mut Ctx, mut body: impl IoBytesStream) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let data = body.next().await.transpose()?;

        Ok(data.unwrap_or_default())
    }
}
