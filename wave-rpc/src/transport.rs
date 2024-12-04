#![allow(unused)]
use async_channel::{Receiver, RecvError, SendError, Sender};
use bytes::{Bytes, BytesMut};
use derive_more::derive::{Display, From};
use futures_lite::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _};
use std::{future::Future, pin::Pin};

use crate::{code::Code, error::RpcError};

pub struct Connection {
    io: Pin<Box<dyn Transport + Send>>,
}

impl Connection {
    pub fn new(io: impl AsyncRead + AsyncWrite + Send + 'static) -> Self {
        Self { io: Box::pin(io) }
    }
}

impl Connection {
    pub async fn process(&mut self, receiver: Receiver<Command>) -> Result<(), std::io::Error> {
        while let Ok(cmd) = receiver.recv().await {
            match cmd {
                Command::Read(mut buf, mut tx) => {
                    self.io.read_exact(&mut buf).await?;
                    if let Err(e) = tx.send(buf.freeze()).await {
                        tracing::debug!("failed to send bytes: {}", e);
                        break;
                    }
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
    Read(BytesMut, Sender<Bytes>),
    Write(Bytes),
    Close,
}

trait Transport: AsyncRead + AsyncWrite {}

impl<T: AsyncRead + AsyncWrite> Transport for T {}

pub struct ConnectionReader {
    sender: Sender<Command>,
}

impl ConnectionReader {
    pub fn new(sender: Sender<Command>) -> Self {
        Self { sender }
    }

    pub async fn read(&mut self, mut buf: BytesMut) -> Result<Bytes, Error> {
        let (tx, rx) = async_channel::bounded(1);
        self.sender.send(Command::Read(buf, tx)).await?;
        let res = rx.recv().await?;
        Ok(res)
    }
}

pub struct ConnectionWriter {
    sender: Sender<Command>,
}

impl ConnectionWriter {
    pub fn new(sender: Sender<Command>) -> Self {
        Self { sender }
    }

    pub async fn write(&mut self, buf: Bytes) -> Result<(), Error> {
        self.sender.send(Command::Write(buf)).await?;
        Ok(())
    }
}

#[derive(Debug, Display, From, derive_more::Error)]
pub enum Error {
    SendError(SendError<Command>),
    ReceiverError(RecvError),
}

impl RpcError for Error {
    fn code(&self) -> Code {
        match self {
            Error::ReceiverError(_) | Error::SendError(_) => Code::InternalServerError,
        }
    }
}
