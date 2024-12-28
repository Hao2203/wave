use std::{
    io::{self, Cursor},
    pin::Pin,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

/// A wrapper around an `AsyncRead` that reads from a buffer first.
#[pin_project::pin_project]
pub struct IoPreHandler<T> {
    buf: Cursor<Vec<u8>>,
    #[pin]
    io: T,
}

impl<T> IoPreHandler<T>
where
    T: AsyncRead + Send + Unpin,
{
    /// Create a new `IoReader` with an internal buffer of `buf_length` bytes.
    ///
    /// The internal buffer is filled with data from the underlying `io` on
    /// creation. The `IoReader` is then positioned at the beginning of the
    /// buffer.
    ///
    /// The `IoReader` will never read more than `buf_length` bytes from the
    /// underlying `io`. If the underlying `io` returns an error or EOF while
    /// attempting to fill the buffer, the `IoReader` will return the same error
    /// or EOF on the next call to `poll_read`.
    pub async fn new(mut io: T, buf_length: usize) -> Result<Self, io::Error> {
        let mut buf = Vec::with_capacity(buf_length);
        io.read_buf(&mut buf).await?;
        let buf = Cursor::new(buf);
        Ok(IoPreHandler { buf, io })
    }
}

impl<T> IoPreHandler<T> {
    pub fn into_inner(self) -> T {
        self.io
    }

    /// Reset the internal buffer to the beginning of the internal buffer.
    pub fn reset(&mut self) {
        self.buf.set_position(0);
    }
}

impl<T> AsyncRead for IoPreHandler<T>
where
    T: AsyncRead + Send,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        let this = self.project();
        let mut reader = this.buf.chain(this.io);
        let reader = Pin::new(&mut reader);
        reader.poll_read(cx, buf)
    }
}

impl<T> AsyncWrite for IoPreHandler<T>
where
    T: AsyncWrite + Send + Unpin,
{
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        let this = self.get_mut();
        let io = Pin::new(&mut this.io);
        io.poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        let this = self.get_mut();
        let io = Pin::new(&mut this.io);
        io.poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        let this = self.get_mut();
        let io = Pin::new(&mut this.io);
        io.poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_io_reader() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        const BUF_LENGTH: usize = 3;
        let io = Cursor::new(&data);
        let mut reader = IoPreHandler::new(io, BUF_LENGTH).await.unwrap();

        let mut buf = [0u8; BUF_LENGTH];
        reader.read_buf(&mut buf.as_mut()).await.unwrap();
        assert_eq!(&buf, &data[..BUF_LENGTH]);

        reader.reset();

        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, data);

        reader.reset();

        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();
        assert_eq!(&buf, &data[..BUF_LENGTH]);
    }
}
