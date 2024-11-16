use crate::message::{FromReader, WriteIn};
use derive_more::derive::Display;
use futures::{io::AsyncReadExt, AsyncWriteExt};
use zerocopy::{Immutable, IntoBytes, TryFromBytes};

#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    Eq,
    PartialEq,
    zerocopy::TryFromBytes,
    zerocopy::IntoBytes,
    Immutable,
)]
#[non_exhaustive]
#[repr(u32)]
pub enum Code {
    Ok,
    IoError,
}

impl Code {
    const SIZE: usize = 4;
    const BUFFER: [u8; Self::SIZE] = [0u8; Self::SIZE];
}

impl FromReader<'_> for Code {
    type Error = std::io::Error;

    async fn from_reader(mut reader: impl futures::AsyncRead + Unpin) -> Result<Self, Self::Error> {
        let mut buf = Code::BUFFER;
        reader.read_exact(&mut buf).await?;
        let code = Code::try_read_from_bytes(&buf).map_err(|_| std::io::ErrorKind::InvalidData)?;
        Ok(code)
    }
}

impl WriteIn for Code {
    type Error = std::io::Error;
    fn write_in<'a>(
        &'a mut self,
        io: &'a mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> futures::future::BoxFuture<'a, Result<(), Self::Error>> {
        let fut = async move {
            io.write_all(self.as_bytes()).await?;
            Ok(())
        };
        Box::pin(fut)
    }
}
