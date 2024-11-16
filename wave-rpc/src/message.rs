use futures::{future::BoxFuture, io::AsyncReadExt, AsyncRead, AsyncWrite, AsyncWriteExt};
use std::future::Future;

pub mod stream;

pub trait FromReader<'a> {
    type Error: core::error::Error + Send;

    fn from_reader(
        reader: impl AsyncRead + Send + Unpin + 'a,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;
}

pub trait WriteIn {
    type Error: core::error::Error + Send;

    fn write_in<'a>(
        &'a mut self,
        io: &'a mut (dyn AsyncWrite + Send + Unpin),
    ) -> BoxFuture<'a, Result<(), Self::Error>>;
}

impl FromReader<'_> for String {
    type Error = std::io::Error;

    async fn from_reader(mut reader: impl AsyncRead + Send + Unpin) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        String::from_utf8(buf).map_err(|_| std::io::ErrorKind::InvalidData.into())
    }
}

impl WriteIn for String {
    type Error = std::io::Error;

    fn write_in<'a>(
        &'a mut self,
        io: &'a mut (dyn AsyncWrite + Send + Unpin),
    ) -> BoxFuture<'a, Result<(), Self::Error>> {
        Box::pin(async move {
            io.write_all(self.as_bytes()).await?;
            Ok(())
        })
    }
}
