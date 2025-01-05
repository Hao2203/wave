use bytes::BytesMut;
use futures_lite::ready;
use std::{
    future::Future,
    io::{self, Cursor},
    pin::{pin, Pin},
    sync::{Arc, Mutex},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

/// A wrapper around an `AsyncRead` that reads from a buffer first.
#[pin_project::pin_project]
pub struct BufConnManager<T> {
    buf: Cursor<BytesMut>,
    #[pin]
    io: T,
}

impl<T> BufConnManager<T> {
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
    pub fn new(io: T, buf_length: usize) -> Self {
        BufConnManager {
            buf: Cursor::new(BytesMut::with_capacity(buf_length)),
            io,
        }
    }

    pub fn into_inner(self) -> T {
        self.io
    }

    pub fn buffer_capacity(&self) -> usize {
        self.buf.get_ref().capacity()
    }

    pub fn buffer_len(&self) -> usize {
        self.buf.get_ref().len()
    }

    /// Reset the internal buffer to the beginning of the internal buffer.
    pub fn reset(&mut self) {
        self.buf.set_position(0);
    }
}

impl<T> AsyncRead for BufConnManager<T>
where
    T: AsyncRead,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        let this = self.project();
        let mut io = this.io;
        if this.buf.get_ref().is_empty() {
            let fut = io.read_buf(this.buf.get_mut());
            ready!(pin!(fut).poll(cx))?;
        }
        let mut reader = this.buf.chain(io);
        pin!(reader).poll_read(cx, buf)
    }
}

impl<T> AsyncWrite for BufConnManager<T>
where
    T: AsyncWrite,
{
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        let this = self.project();
        this.io.poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        let this = self.project();
        this.io.poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        let this = self.project();
        this.io.poll_shutdown(cx)
    }
}

#[derive(Debug)]
pub struct Cloneable<T> {
    pub value: Arc<Mutex<T>>,
}

impl<T> Clone for Cloneable<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl<T> Cloneable<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(Mutex::new(value)),
        }
    }
}

impl<T> AsyncRead for Cloneable<T>
where
    T: AsyncRead + std::marker::Unpin,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        let mut value = self.value.lock().unwrap();
        let value = Pin::new(&mut *value);
        value.poll_read(cx, buf)
    }
}

impl<T> AsyncWrite for Cloneable<T>
where
    T: AsyncWrite + std::marker::Unpin,
{
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        let mut value = self.value.lock().unwrap();
        let value = Pin::new(&mut *value);
        value.poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        let mut value = self.value.lock().unwrap();
        let value = Pin::new(&mut *value);
        value.poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        let mut value = self.value.lock().unwrap();
        let value = Pin::new(&mut *value);
        value.poll_shutdown(cx)
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
        let mut reader = BufConnManager::new(io, BUF_LENGTH);

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
