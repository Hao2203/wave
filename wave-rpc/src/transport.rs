use tokio::io::{AsyncRead, AsyncWrite};

use std::{future::Future, io::Error as IoError};

pub type IoResult<T, E = IoError> = std::result::Result<T, E>;

pub trait Transport {
    fn from_io(
        io: impl AsyncRead + Send + Sync + Unpin,
    ) -> impl Future<Output = IoResult<Option<Self>>> + Send
    where
        Self: Sized;

    fn write_into(
        &mut self,
        io: impl AsyncWrite + Send + Sync + Unpin,
    ) -> impl Future<Output = IoResult<()>> + Send;
}
