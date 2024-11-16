use super::{FromReader, WriteIn};
use async_trait::async_trait;
use derive_more::derive::Display;
use futures::{
    pin_mut,
    stream::{self, BoxStream},
    AsyncRead, AsyncWrite, StreamExt,
};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Stream<'a, T>
where
    T: Send,
{
    stream: StreamInner<'a, T>,
}

#[async_trait::async_trait]
impl<'a, T> FromReader<'a> for Stream<'a, T>
where
    T: Send,
{
    type Error = std::io::Error;

    async fn from_reader(reader: impl AsyncRead + Send + Unpin + 'a) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Stream {
            stream: StreamInner::Reader(Box::new(reader)),
        })
    }
}

#[async_trait]
impl<T> WriteIn for Stream<'_, T>
where
    T: Send + WriteIn + for<'a> FromReader<'a>,
{
    type Error = std::io::Error;

    async fn write_in(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error> {
        while let Some(item) = self.stream.next().await {
            item.unwrap().write_in(io).await.unwrap();
        }
        Ok(())
    }
}

impl<T> stream::Stream for StreamInner<'_, T>
where
    T: Send + for<'b> FromReader<'b>,
{
    type Item = Result<T, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            Self::Reader(reader) => {
                let io = reader;
                let fut = async move {
                    let item = T::from_reader(io)
                        .await
                        .map_err(|_| Error::Io(std::io::ErrorKind::BrokenPipe.into()));
                    Some(item)
                };
                pin_mut!(fut);
                fut.poll(cx)
            }
            Self::Stream(stream) => stream.poll_next_unpin(cx),
        }
    }
}

pub enum StreamInner<'a, T> {
    Reader(Box<dyn AsyncRead + Send + Unpin + 'a>),
    Stream(BoxStream<'a, Result<T, Error>>),
}

#[derive(Debug, Display, derive_more::Error)]
pub enum Error {
    #[error(ignore)]
    Io(std::io::Error),
}
