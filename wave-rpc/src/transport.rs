#![allow(unused)]
use crate::{code::Code, error::RpcError};
use async_channel::{Receiver, RecvError, SendError, Sender};
use bytes::Buf;
use derive_more::derive::{Display, From};
use futures_lite::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _};
use std::{future::Future, io, pin::Pin, sync::Arc};

pub struct Connection {
    io: Pin<Box<dyn Transport>>,
}

impl Connection {
    pub fn new(io: impl Transport + 'static) -> Self {
        Self { io: Box::pin(io) }
    }
}

impl Connection {
    pub async fn process(mut self, receiver: Receiver<Command>) -> Result<(), Error> {
        while let Ok(cmd) = receiver.recv().await {
            match cmd {
                Command::Read(len, mut tx) => {
                    let mut buf = Vec::with_capacity(len);
                    self.io.read_exact(&mut buf).await?;

                    tx.send(buf)?
                }
                Command::Write(buf) => {
                    self.io.write_all(&buf).await?;
                }
                Command::Close => {
                    break;
                }
            }
        }
        Ok(())
    }
}

pub enum Command {
    Read(usize, oneshot::Sender<Vec<u8>>),
    Write(Arc<[u8]>),
    Close,
}

pub trait Transport: AsyncRead + AsyncWrite + Send + Sync + Unpin {}

impl<T> Transport for T where T: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static {}

#[derive(Debug, Clone)]
pub struct ConnectionManager {
    sender: Sender<Command>,
}

impl ConnectionManager {
    pub fn new(sender: Sender<Command>) -> Self {
        Self { sender }
    }

    /// Reads exactly `len` bytes from the underlying connection.
    /// Not guaranteed to return exactly `len` bytes.
    /// Returns an error if the underlying connection is closed.
    pub async fn read(&self, len: usize) -> Result<Vec<u8>, Error> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(Command::Read(len, tx)).await?;
        let res = rx.await?;
        Ok(res)
    }

    pub async fn get_u8(&self) -> Result<u8, Error> {
        let buf = self.read(1).await?;

        if buf.len() != 1 {
            return Err(Error::UnexpectedDataSize(buf.len()));
        }

        Ok(buf[0])
    }

    pub async fn get_u32(&self) -> Result<u32, Error> {
        let buf = self.read(4).await?;

        if buf.len() != 4 {
            return Err(Error::UnexpectedDataSize(buf.len()));
        }

        Ok(buf.as_slice().get_u32_le())
    }

    pub async fn write(&self, data: Arc<[u8]>) -> Result<(), Error> {
        self.sender.send(Command::Write(data)).await?;
        Ok(())
    }

    pub async fn write_u8(&self, data: u8) -> Result<(), Error> {
        self.write(Arc::from([data])).await
    }

    pub async fn write_u32(&self, data: u32) -> Result<(), Error> {
        self.write(Arc::from(data.to_le_bytes())).await
    }

    pub async fn close(self) -> Result<(), Error> {
        self.sender.send(Command::Close).await?;
        Ok(())
    }
}

#[derive(Debug, Display, From, derive_more::Error)]
pub enum Error {
    SendCommandError,
    ReceiverError(async_channel::RecvError),
    OneshotReceiverEror(oneshot::RecvError),
    OneshotSendError(oneshot::SendError<Vec<u8>>),
    #[from(ignore)]
    UnexpectedDataSize(#[error(not(source))] usize),
    Io(io::Error),
}

impl<T> From<async_channel::SendError<T>> for Error {
    fn from(_: async_channel::SendError<T>) -> Self {
        Error::SendCommandError
    }
}

impl RpcError for Error {
    fn code(&self) -> Code {
        match self {
            Self::Io(e) => e.code(),
            _ => Code::InternalServerError,
        }
    }
}
