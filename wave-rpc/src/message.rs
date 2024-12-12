use bytes::Bytes;
use futures_lite::{stream::StreamExt, Stream};
use std::{error::Error, future::Future, io};

#[cfg(feature = "bincode")]
pub mod bincode;
pub mod stream;

pub trait BytesStream: Stream<Item = Result<Bytes, Self::Error>> + Unpin + Send + 'static {
    type Error: Error;
}

impl<T, E> BytesStream for T
where
    T: Stream<Item = Result<Bytes, E>> + Unpin + Send + 'static,
    E: Error,
{
    type Error = E;
}

pub trait FromStream<Ctx> {
    type Error: Error;

    fn from_stream(
        ctx: &mut Ctx,
        body: impl BytesStream<Error = io::Error>,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;
}

pub trait IntoStream {
    type Error: Error;
    fn into_stream(self) -> impl BytesStream<Error = Self::Error>;
}

impl<Ctx: Send> FromStream<Ctx> for Bytes {
    type Error = io::Error;

    async fn from_stream(
        _ctx: &mut Ctx,
        mut body: impl BytesStream<Error = io::Error>,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let data = body.next().await.transpose()?;

        Ok(data.unwrap_or_default())
    }
}

// impl IntoStream for Bytes {
//     type Error = io::Error;
//     fn into_stream(self) -> impl BytesStream<Error = io::Error> {
//         stream::once(self)
//     }
// }
