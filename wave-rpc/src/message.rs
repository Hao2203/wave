use futures::{future::BoxFuture, AsyncRead, AsyncWrite};
use std::future::Future;

pub mod stream;

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
