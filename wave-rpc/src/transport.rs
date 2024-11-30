#![allow(unused)]
use async_channel::{Receiver, Sender};
use bytes::Bytes;
use futures_lite::{AsyncRead, AsyncWrite};
use std::future::Future;

pub struct Connection<T> {
    io: T,
}

impl<T> Connection<T> {
    pub fn new(io: T) -> Self {
        Self { io }
    }
}

impl<T> Connection<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn process(&mut self, receiver: Receiver<Command>) -> Result<(), std::io::Error> {
        while let Ok(cmd) = receiver.recv().await {
            match cmd {
                Command::Read(mut tx) => {
                    let bytes = self.read_bytes().await?;
                    if let Err(e) = tx.send(bytes).await {
                        tracing::debug!("failed to send bytes: {}", e);
                        break;
                    }
                }
                Command::Write(buf) => {
                    self.write_bytes(buf).await?;
                }
                Command::Close => {
                    break;
                }
            }
        }
        Ok(())
    }

    pub async fn read_bytes(&mut self) -> Result<Bytes, std::io::Error> {
        todo!()
    }

    pub async fn write_bytes(&mut self, bytes: Bytes) -> Result<(), std::io::Error> {
        todo!()
    }
}

pub enum Command {
    Read(Sender<Bytes>),
    Write(Bytes),
    Close,
}
