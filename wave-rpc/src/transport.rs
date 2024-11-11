use async_stream::stream;
use async_trait::async_trait;
use futures::{stream::BoxStream, StreamExt};
use std::{future::Future, io::Error as IoError, pin::Pin};
use tokio::io::{AsyncRead, AsyncWrite};

pub type IoResult<T, E = IoError> = std::result::Result<T, E>;

#[async_trait]
pub trait Transport<'a> {
    type Error;
    async fn from_reader(
        io: impl AsyncRead + Send + Sync + Unpin + 'a,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized + 'a;

    async fn write_into(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error>;
}

pub struct Stream<'a, T> {
    inner: BoxStream<'a, T>,
}

#[async_trait]
impl<'a, T> Transport<'a> for Stream<'a, Result<T, <T as Transport<'_>>::Error>>
where
    T: Send + for<'b> Transport<'b, Error: Send>,
{
    type Error = <T as Transport<'a>>::Error;
    async fn from_reader(
        mut io: impl AsyncRead + Send + Sync + Unpin + 'a,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized + 'a,
    {
        let stream = stream! {
            while let Ok(Some(item)) = T::from_reader(&mut io).await {
                yield Ok(item)
            }
        };
        Ok(Some(Stream {
            inner: Box::pin(stream),
        }))
    }

    async fn write_into(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error> {
        todo!()
    }
}
