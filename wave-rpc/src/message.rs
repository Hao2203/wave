use bytes::Bytes;
use futures_lite::{stream::StreamExt, Stream};
use std::{convert::Infallible, error::Error, future::Future, io};

#[cfg(feature = "bincode")]
pub mod bincode;
pub mod stream;

pub trait BytesStream: Stream<Item = Bytes> + Send + Unpin + 'static {}

impl<T> BytesStream for T where T: Stream<Item = Bytes> + Send + Unpin + 'static {}

pub trait TryBytesStream:
    Stream<Item = Result<Bytes, Self::Error>> + Unpin + Send + 'static
{
    type Error: Error;
}

impl<T, E> TryBytesStream for T
where
    T: Stream<Item = Result<Bytes, E>> + Unpin + Send + 'static,
    E: Error,
{
    type Error = E;
}

pub trait FromStream {
    type Error: Error;

    fn from_stream(
        stream: impl BytesStream,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;
}

pub trait IntoStream {
    type Error: Error;
    fn into_stream(self) -> impl TryBytesStream<Error = Self::Error>;
}

impl FromStream for Bytes {
    type Error = Infallible;

    async fn from_stream(mut stream: impl BytesStream) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let data = stream.next().await;

        Ok(data.unwrap_or_default())
    }
}

impl IntoStream for Bytes {
    type Error = Infallible;
    fn into_stream(self) -> impl TryBytesStream<Error = Infallible> {
        futures_lite::stream::once(Ok(self))
    }
}
