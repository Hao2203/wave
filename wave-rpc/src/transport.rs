use async_stream::stream;
use async_trait::async_trait;
use futures::{stream::BoxStream, StreamExt};
use std::{
    convert::Infallible,
    fmt::{Debug, Display},
};
use tokio::io::{AsyncRead, AsyncWrite};

#[async_trait]
pub trait FromReader<'a> {
    type Error: Display + Debug + core::error::Error + Send;
    async fn from_reader(
        io: impl AsyncRead + Send + Unpin + 'a,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized;
}

#[async_trait]
pub trait WriteIn {
    type Error: Display + Debug + core::error::Error + Send;

    async fn write_in(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error>;
}

pub struct Stream<'a, T> {
    inner: BoxStream<'a, T>,
}

#[async_trait]
impl<'a, T> FromReader<'a> for Stream<'a, T>
where
    T: Send + for<'b> FromReader<'b> + 'a,
{
    type Error = Infallible;

    async fn from_reader(
        mut io: impl AsyncRead + Send + Unpin + 'a,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized,
    {
        let stream = stream! {
            while let Ok(Some(item)) = T::from_reader(&mut io).await {
                yield item
            }
        };
        Ok(Some(Stream {
            inner: stream.boxed(),
        }))
    }
}

#[async_trait]
impl<'a, T> WriteIn for Stream<'a, T>
where
    T: Send + WriteIn + 'a,
{
    type Error = T::Error;

    async fn write_in(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error> {
        while let Some(mut item) = self.inner.next().await {
            item.write_in(io).await?
        }
        Ok(())
    }
}

#[async_trait]
impl<'a, T, E> FromReader<'a> for Result<T, E>
where
    T: FromReader<'a, Error = E> + Send,
{
    type Error = Infallible;

    async fn from_reader(
        io: impl AsyncRead + Send + Unpin + 'a,
    ) -> Result<Option<Self>, Self::Error> {
        Ok(T::from_reader(io).await.transpose())
    }
}

#[async_trait]
impl<'a, T, E> WriteIn for Result<T, E>
where
    T: WriteIn<Error = E> + Send,
    E: Send + core::error::Error,
{
    type Error = E;

    async fn write_in(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error> {
        if let Ok(item) = self {
            item.write_in(io).await?;
        }
        Ok(())
    }
}
