pub use crate::error::{Error, Result};
use async_channel::Receiver;
use futures_lite::{Stream, StreamExt as _};
use std::{net::SocketAddr, ops::Not, pin::Pin};
use tokio::io::{AsyncRead, AsyncWrite};

pub mod error;
pub mod socks5;

pub trait ProxyBuilder {
    type Stream: Stream<Item = Result<Incoming>> + Send + Unpin + 'static;
    fn build(self) -> impl std::future::Future<Output = Result<Self::Stream>> + Send;
}

pub trait Io: AsyncRead + AsyncWrite + Send + Unpin {}

impl<T: AsyncRead + AsyncWrite + Send + Unpin> Io for T {}

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
    pub target: Target,
    pub io: Box<dyn Io>,
}

impl Incoming {
    pub fn new(target: impl Into<Target>, io: impl Io + 'static) -> Self {
        Self {
            target: target.into(),
            io: Box::new(io),
        }
    }
}

pub enum Target {
    Ip(SocketAddr),
    Domain(String, u16),
}
