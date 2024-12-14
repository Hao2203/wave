use derive_more::derive::Display;
use futures_lite::{AsyncRead, AsyncWrite};
use parking_lot::Mutex;
use std::{io, pin::Pin, sync::Arc, task::Poll};

pub trait Transport: AsyncRead + AsyncWrite + Send + Sync + Unpin {}

impl<T> Transport for T where T: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static {}

/// A connection manager that can be get reader and writer
pub struct Connection<T> {
    io: Arc<Mutex<Option<T>>>,
}

impl<T> Connection<T> {
    pub fn new(io: T) -> Self {
        Self {
            io: Arc::new(Mutex::new(Some(io))),
        }
    }

    /// Stop and return the underlying connection.
    ///
    /// This method is used to stop the connection manager and return the underlying connection.
    /// It is used to stop the connection manager and return the underlying connection to the caller.
    ///
    /// # Panics
    ///
    /// Panics if the connection manager has already been stopped.
    ///
    pub fn stop(self) -> T {
        self.io.lock().take().expect("unexpected double stop")
    }

    fn clone(&self) -> Self {
        Self {
            io: self.io.clone(),
        }
    }
}

impl<T> Connection<T>
where
    T: AsyncRead + Unpin,
{
    pub fn get_reader(&self) -> ConnectionReader<T> {
        ConnectionReader(self.clone())
    }
}

impl<T> Connection<T>
where
    T: AsyncWrite + Unpin,
{
    pub fn get_writer(&self) -> ConnectionWriter<T> {
        ConnectionWriter(self.clone())
    }
}

impl<T> AsyncRead for Connection<T>
where
    T: AsyncRead + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let mut reader = self.io.lock();
        match *reader {
            None => Poll::Ready(Err(Error::ConnectionIsStopped.into())),
            Some(ref mut reader) => {
                let reader = Pin::new(&mut *reader);
                reader.poll_read(cx, buf)
            }
        }
    }
}

impl<T> AsyncWrite for Connection<T>
where
    T: AsyncWrite + Unpin,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let mut writer = self.io.lock();
        match *writer {
            None => Poll::Ready(Err(Error::ConnectionIsStopped.into())),
            Some(ref mut writer) => {
                let writer = Pin::new(&mut *writer);
                writer.poll_write(cx, buf)
            }
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut writer = self.io.lock();
        match *writer {
            None => Poll::Ready(Err(Error::ConnectionIsStopped.into())),
            Some(ref mut writer) => {
                let writer = Pin::new(&mut *writer);
                writer.poll_flush(cx)
            }
        }
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut writer = self.io.lock();
        match *writer {
            None => Poll::Ready(Err(Error::ConnectionIsStopped.into())),
            Some(ref mut writer) => {
                let writer = Pin::new(&mut *writer);
                writer.poll_close(cx)
            }
        }
    }
}

pub struct ConnectionReader<T>(Connection<T>);

impl<T> AsyncRead for ConnectionReader<T>
where
    T: AsyncRead + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let reader = Pin::new(&mut self.get_mut().0);
        reader.poll_read(cx, buf)
    }
}

pub struct ConnectionWriter<T>(Connection<T>);

impl<T> AsyncWrite for ConnectionWriter<T>
where
    T: AsyncWrite + Unpin,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let writer = Pin::new(&mut self.get_mut().0);
        writer.poll_write(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let writer = Pin::new(&mut self.get_mut().0);
        writer.poll_flush(cx)
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let writer = Pin::new(&mut self.get_mut().0);
        writer.poll_close(cx)
    }
}

#[derive(Debug, Display, derive_more::Error)]
pub enum Error {
    ConnectionIsStopped,
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        io::Error::new(io::ErrorKind::Other, err)
    }
}
