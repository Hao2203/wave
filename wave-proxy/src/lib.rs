pub use crate::error::{Error, ErrorKind, Result};
use async_channel::Receiver;
use futures_lite::{ready, Stream, StreamExt as _};
use std::{future::Future, net::SocketAddr, pin::Pin, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite};

pub mod error;
// pub mod socks5;
#[cfg(test)]
mod tests;

pub trait ProxyBuilder {
    type Stream: Stream<Item = Result<Incoming>> + Send + Unpin + 'static;
    fn build(self) -> impl Future<Output = Result<Self::Stream>> + Send;
}

#[async_trait::async_trait]
pub trait Proxy {
    async fn info(&self, conn: &mut dyn Io) -> Result<ConnectionInfo>;
}

pub trait Io: AsyncRead + AsyncWrite + Send + Unpin {}

impl<T: AsyncRead + AsyncWrite + Send + Unpin> Io for T {}

type PinReceiver = Pin<Box<Receiver<Result<Box<dyn Io>>>>>;

#[derive()]
pub struct ProxyServer {
    receiver: PinReceiver,
    proxy: Arc<dyn Proxy>,
}

impl ProxyServer {
    // pub async fn new(builder: impl ProxyBuilder) -> Result<Self> {
    //     let (sender, receiver) = async_channel::unbounded();
    //     let mut stream = builder.build().await?;
    //     tokio::spawn(async move {
    //         while let Some(res) = stream.next().await {
    //             if sender.is_closed() {
    //                 break;
    //             }
    //             let sender = sender.clone();
    //             tokio::spawn(async move {
    //                 if let Err(e) = sender.send(res).await {
    //                     tracing::warn!("failed to send incoming: {}", e);
    //                 }
    //             });
    //         }
    //     });
    //     Ok(Self {
    //         inner: Box::pin(receiver),
    //     })
    // }
}

impl Stream for ProxyServer {
    type Item = Result<Incoming>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let io = ready!(self.as_mut().receiver.poll_next(cx));
        match io {
            Some(Ok(io)) => {
                let proxy = self.get_mut().proxy.clone();
                std::task::Poll::Ready(Some(Ok(Incoming { proxy, io })))
            }
            Some(Err(e)) => std::task::Poll::Ready(Some(Err(e))),
            None => std::task::Poll::Ready(None),
        }
    }
}

pub struct ConnectionInfo {
    pub target: Target,
}

pub struct Incoming {
    proxy: Arc<dyn Proxy>,
    io: Box<dyn Io>,
}

impl Incoming {
    pub async fn extract_info(&mut self) -> Result<ConnectionInfo> {
        self.proxy.info(&mut *self.io).await
    }
}

#[derive(Debug, Clone)]
pub enum Target {
    Ip(SocketAddr),
    Domain(String, u16),
}
