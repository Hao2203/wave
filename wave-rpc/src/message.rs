use futures::{
    future::BoxFuture,
    pin_mut,
    stream::{self},
    AsyncRead, AsyncWrite, StreamExt,
};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub trait Message<'a> {
    type Error: core::error::Error + Send;

    fn from_reader(
        reader: impl AsyncRead + Send + Unpin + 'a,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;

    fn write_in<'b>(
        &'b mut self,
        io: &'b mut (dyn AsyncWrite + Send + Unpin),
    ) -> BoxFuture<'b, Result<(), Self::Error>>;
}

pub struct Stream<'a, T> {
    reader: Box<dyn AsyncRead + Send + Unpin + 'a>,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<'a, T, E> Message<'a> for Stream<'a, T>
where
    T: Send + for<'b> Message<'b, Error = E>,
    E: core::error::Error + Send,
{
    type Error = E;

    async fn from_reader(reader: impl AsyncRead + Send + Unpin + 'a) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Stream {
            reader: Box::new(reader),
            _marker: std::marker::PhantomData,
        })
    }

    fn write_in<'b>(
        &'b mut self,
        io: &'b mut (dyn AsyncWrite + Send + Unpin),
    ) -> BoxFuture<'b, Result<(), Self::Error>> {
        Box::pin(async {
            while let Some(item) = self.next().await {
                item?.write_in(io).await?;
            }
            Ok(())
        })
    }
}

impl<'a, T, E> stream::Stream for Stream<'a, T>
where
    T: Send + for<'b> Message<'b, Error = E>,
{
    type Item = Result<T, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let io = &mut self.get_mut().reader;
        let fut = async move {
            let item = T::from_reader(io).await;
            Some(item)
        };
        pin_mut!(fut);
        fut.poll(cx)
    }
}
