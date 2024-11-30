#![allow(unused)]
use super::{FromReader, SendTo};
use async_trait::async_trait;
use derive_more::derive::Display;
use futures_lite::{
    stream::{self, Boxed},
    AsyncRead, AsyncWrite, StreamExt,
};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Stream<T> {
    receiver: async_channel::Receiver<T>,
}

impl<T> FromReader for Stream<T>
where
    T: Send + FromReader + 'static,
{
    type Error = std::io::Error;

    async fn from_reader(reader: &mut (impl AsyncRead + Send + Unpin)) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let (sender, receiver) = async_channel::bounded(1);
        let item = T::from_reader(reader).await.unwrap();
        tokio::spawn(async move {
            sender.send(item).await.unwrap();
        });
        Ok(Stream { receiver })
    }
}
