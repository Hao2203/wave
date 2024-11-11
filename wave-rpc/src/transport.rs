use async_stream::stream;
use async_trait::async_trait;
use futures::{stream::BoxStream, StreamExt};
use std::{
    fmt::{Debug, Display},
    future::Future,
    io::{self, Error as IoError},
    pin::Pin,
};
use tokio::io::{AsyncRead, AsyncWrite};

pub type IoResult<T, E = IoError> = std::result::Result<T, E>;

#[async_trait]
pub trait Transport<'a> {
    type Error: Display + Debug + core::error::Error + Send;
    async fn from_reader(
        io: impl AsyncRead + Send + Sync + Unpin + 'a,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized;

    async fn write_into(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error>;
}

pub struct Stream<'a, T> {
    inner: BoxStream<'a, Result<T, io::Error>>,
}

#[async_trait]
impl<'a, T> Transport<'a> for Stream<'a, T>
where
    T: Send + for<'b> Transport<'b> + 'a,
{
    type Error = io::Error;
    async fn from_reader(
        mut io: impl AsyncRead + Send + Sync + Unpin + 'a,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized,
    {
        let stream = stream! {
            loop  {
                let item = T::from_reader(&mut io).await.transpose();
                match item {
                    Some(item) => yield item.map_err(|_e| {
                        io::ErrorKind::NotFound.into()
                    }),
                    None => {}
                }
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
