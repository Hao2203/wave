use crate::error::Result;
use async_channel::Receiver;
use futures_lite::{Stream, StreamExt as _};
use std::{net::SocketAddr, ops::Not, pin::Pin};
use tokio::io::{AsyncRead, AsyncWrite};

pub mod error;

type Inner = Pin<Box<Receiver<Result<Incoming>>>>;

#[derive(Clone)]
pub struct Proxy {
    inner: Inner,
}

impl Proxy {
    pub async fn new(builder: impl ProxyBuilder) -> Result<Self> {
        let (sender, receiver) = async_channel::unbounded();
        let mut stream = builder.build().await?;
        tokio::spawn(async move {
            while let Some(res) = stream.next().await {
                if sender.is_closed().not() {
                    if let Err(e) = sender.send(res).await {
                        tracing::warn!("failed to send incoming: {}", e);
                    }
                }
            }
        });
        Ok(Self {
            inner: Box::pin(receiver),
        })
    }
}

impl Stream for Proxy {
    type Item = Result<Incoming>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.get_mut().inner.poll_next(cx)
    }
}

pub struct Incoming {
    pub target_addr: TargetAddr,
    pub io: Box<dyn Io>,
}

pub enum TargetAddr {
    Ip(SocketAddr),
    Domain(String, u16),
}

pub trait ProxyBuilder {
    type Stream: Stream<Item = Result<Incoming>> + Send + Unpin + 'static;
    fn build(self) -> impl std::future::Future<Output = Result<Self::Stream>> + Send;
}

pub trait Io: AsyncRead + AsyncWrite + Send + Unpin {}
