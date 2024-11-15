use super::Message;
use futures::{
    future::BoxFuture,
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
    T: Send + Message<'a>,
{
    stream: StreamInner<'a, T>,
}

impl<'a, T, E> Message<'a> for Stream<'a, T>
where
    T: Send + for<'b> Message<'b, Error = E> + 'a,
    E: core::error::Error + Send + 'a,
{
    type Error = std::io::Error;

    async fn from_reader(reader: impl AsyncRead + Send + Unpin + 'a) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            stream: StreamInner::Reader(Box::new(reader)),
        })
    }

    fn write_in(
        &'a mut self,
        io: &'a mut (dyn AsyncWrite + Send + Unpin),
    ) -> BoxFuture<'a, Result<(), Self::Error>> {
        Box::pin(async move {
            while let Some(item) = self.stream.next().await {
                item.unwrap().write_in(io).await.unwrap();
            }
            Ok(())
        })
    }
}

impl<'a, T, E> stream::Stream for StreamInner<'a, T>
where
    T: Send + for<'b> Message<'b, Error = E>,
{
    type Item = Result<T, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            Self::Reader(reader) => {
                let io = reader;
                let fut = async move {
                    let item = T::from_reader(io).await;
                    Some(item)
                };
                pin_mut!(fut);
                fut.poll(cx)
            }
            Self::Stream(stream) => stream.poll_next_unpin(cx),
        }
    }
}

pub enum StreamInner<'a, T>
where
    T: Send + Message<'a>,
{
    Reader(Box<dyn AsyncRead + Send + Unpin + 'a>),
    Stream(BoxStream<'a, Result<T, T::Error>>),
}
