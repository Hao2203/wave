use futures::future::BoxFuture;
use std::{future::Future, io::Error as IoError};
use tokio::io::{AsyncRead, AsyncWrite};

pub type IoResult<T, E = IoError> = std::result::Result<T, E>;

pub trait Transport<'a> {
    type Error;
    fn from_reader(
        io: impl AsyncRead + Send + Sync + Unpin + 'a,
    ) -> impl Future<Output = Result<Option<Self>, Self::Error>> + Send
    where
        Self: Sized;

    fn write_into(
        &'a mut self,
        io: &'a mut (dyn AsyncWrite + Send + Unpin),
    ) -> BoxFuture<'a, Result<(), Self::Error>>;
}
