// #![allow(unused)]
use crate::ALPN;
use bytes::BytesMut;
use futures_lite::FutureExt;
use iroh::{
    endpoint::{RecvStream, SendStream},
    Endpoint, NodeId,
};
use std::{net::SocketAddr, pin::pin, str::FromStr};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};
use tracing::info;
use wave_proxy::{
    protocol::socks5::{
        types::{ConnectRequest, ConnectedStatus, HandshakeRequest},
        NoAuthHandshake, Transmit,
    },
    Address,
};

#[cfg(test)]
mod tests;

pub struct Client {
    listener: tokio::net::TcpListener,
    endpoint: Endpoint,
}

impl Client {
    pub async fn run(self) -> anyhow::Result<()> {
        loop {
            let (stream, local) = self.listener.accept().await?;
            let endpoint = self.endpoint.clone();
            tokio::spawn(async move {
                let upstream_address = stream
                    .peer_addr()
                    .inspect_err(|e| tracing::error!("Get peer address failed: {}", e))
                    .expect("Get peer address failed");
                let handler = Handler {
                    local,
                    endpoint,
                    upstream_address,
                    upstream: Stream::Tcp(stream),
                    downstream: None,
                };
                handler
                    .handle()
                    .await
                    .inspect_err(|e| {
                        tracing::error!("handle incomming error: {}", e);
                    })
                    .expect("handle incomming failed");
            });
        }
    }
}

struct Handler {
    local: SocketAddr,
    upstream_address: SocketAddr,
    endpoint: Endpoint,
    upstream: Stream,
    downstream: Option<(Address, Stream)>,
}

impl Handler {
    async fn handle(mut self) -> anyhow::Result<()> {
        let socks5 = NoAuthHandshake::new(self.local, self.upstream_address);

        let mut buf = BytesMut::with_capacity(1024);
        self.upstream.read_buf(&mut buf).await?;
        let req = HandshakeRequest::decode(&mut buf)?.unwrap();
        let (transmit, socks5) = socks5.handshake(req);
        self.send_transmit(transmit).await?;

        self.upstream.read_buf(&mut buf).await?;

        let req = ConnectRequest::decode(&mut buf)?.unwrap();
        info!("Try to connect to {}", req.target);
        let mut socks5 = match self.set_downstream(req.target.clone()).await {
            Ok(()) => {
                let (transmit, socks5) = socks5?.connect(req, ConnectedStatus::Succeeded);
                self.send_transmit(transmit).await?;
                Ok(socks5?)
            }
            Err(e) => {
                let (transmit, _) = socks5?.connect(req, ConnectedStatus::HostUnreachable);
                self.send_transmit(transmit).await?;
                Err(e)
            }
        }?;

        buf.clear();
        buf.reserve(8 * 1024);
        let mut buf2 = BytesMut::with_capacity(8 * 1024);
        loop {
            if let Some((addr, stream)) = self.downstream.as_mut() {
                let (addr, data) = async {
                    stream.read_buf(&mut buf).await?;
                    anyhow::Ok((addr.clone(), buf.split().freeze()))
                }
                .race(async {
                    self.upstream.read_buf(&mut buf2).await?;
                    Ok((self.upstream_address.into(), buf2.split().freeze()))
                })
                .await?;
                let transmit = socks5.relay(addr, data);
                self.send_transmit(transmit).await?;
            }
        }
    }

    async fn set_downstream(&mut self, addr: Address) -> anyhow::Result<()> {
        let stream = match &addr {
            Address::Ip(ip) => {
                let stream = TcpStream::connect(ip).await?;
                Stream::Tcp(stream)
            }
            Address::Domain(domain, port) => match NodeId::from_str(domain) {
                Ok(node_id) => {
                    let conn = self.endpoint.connect(node_id, ALPN).await?;
                    let stream = conn.open_bi().await?;
                    Stream::Iroh(stream.0, stream.1)
                }
                Err(_e) => {
                    let stream = TcpStream::connect((domain.as_ref(), *port)).await?;
                    Stream::Tcp(stream)
                }
            },
        };
        self.downstream = Some((addr, stream));
        Ok(())
    }

    async fn connect(&mut self, address: Address) -> anyhow::Result<&mut Stream> {
        if Address::Ip(self.upstream_address) == address {
            return Ok(&mut self.upstream);
        }

        if let Some((addr, stream)) = self.downstream.as_mut() {
            if *addr == address {
                return Ok(stream);
            }
        }
        Err(anyhow::anyhow!("downstream address mismatch"))
    }

    async fn send_transmit(
        &mut self,
        Transmit { to, mut data, .. }: Transmit,
    ) -> anyhow::Result<()> {
        let stream = self.connect(to).await?;

        stream.write_all_buf(&mut data).await?;

        Ok(())
    }
}

pub enum Stream {
    Iroh(SendStream, RecvStream),
    Tcp(TcpStream),
}

impl AsyncRead for Stream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Stream::Iroh(_, recv_stream) => pin!(recv_stream).poll_read(cx, buf),
            Stream::Tcp(stream) => pin!(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            Stream::Iroh(send_stream, _) => pin!(send_stream).poll_write(cx, buf),
            Stream::Tcp(stream) => pin!(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Stream::Iroh(send_stream, _) => pin!(send_stream).poll_flush(cx),
            Stream::Tcp(stream) => pin!(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Stream::Iroh(send_stream, _) => pin!(send_stream).poll_shutdown(cx),
            Stream::Tcp(stream) => pin!(stream).poll_shutdown(cx),
        }
    }
}
