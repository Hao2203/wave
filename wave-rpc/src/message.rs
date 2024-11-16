use async_trait::async_trait;
use futures::{io::AsyncReadExt, AsyncRead, AsyncWrite, AsyncWriteExt};

pub mod stream;

#[async_trait]
pub trait FromReader<'a> {
    type Error: core::error::Error + Send;

    async fn from_reader(reader: impl AsyncRead + Send + Unpin + 'a) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

#[async_trait]
pub trait WriteIn {
    type Error: core::error::Error + Send;

    async fn write_in(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error>;
}

#[async_trait]
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

#[async_trait]
impl WriteIn for String {
    type Error = std::io::Error;

    async fn write_in(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error> {
        io.write_all(self.as_bytes()).await?;
        Ok(())
    }
}
