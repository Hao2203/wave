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
                    let mut buf = Vec::new();
                    todo!();
                    tx.send(Bytes::from(buf)).await;
                }
                Command::Write(buf) => {
                    todo!();
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
    Read(Sender<Bytes>),
    Write(Bytes),
    Close,
}
