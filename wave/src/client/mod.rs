// #![allow(unused)]
use crate::{Stream, ALPN};
use bytes::BytesMut;
use futures_lite::FutureExt;
use iroh::Endpoint;
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};
use tracing::{debug, info, warn};
use wave_core::{NodeId, Server};
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
    server: Arc<Server>,
    listener: tokio::net::TcpListener,
    endpoint: Endpoint,
}

impl Client {
    pub async fn new<A: ToSocketAddrs>(
        server: Arc<Server>,
        bind: A,
        endpoint: Endpoint,
    ) -> Result<Self, std::io::Error> {
        let listener = tokio::net::TcpListener::bind(bind).await?;
        Ok(Self {
            server,
            listener,
            endpoint,
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        loop {
            let (stream, local) = self.listener.accept().await?;
            let endpoint = self.endpoint.clone();
            let server = self.server.clone();
            tokio::spawn(async move {
                let upstream_address = stream
                    .peer_addr()
                    .inspect_err(|e| tracing::error!("Get peer address failed: {}", e))
                    .expect("Get peer address failed");
                let handler = Handler {
                    server,
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
    server: Arc<Server>,
    local: SocketAddr,
    upstream_address: SocketAddr,
    endpoint: Endpoint,
    upstream: Stream,
    downstream: Option<(Address, Stream)>,
}

impl Handler {
    async fn handle(mut self) -> anyhow::Result<()> {
        info!("Connect from {}", self.upstream_address);

        let socks5 = NoAuthHandshake::new(self.local, self.upstream_address);

        let mut buf = BytesMut::with_capacity(1024);
        self.upstream.read_buf(&mut buf).await?;
        let req = HandshakeRequest::decode(&mut buf)?.unwrap();
        let (transmit, socks5) = socks5.handshake(req);
        self.send_transmit(transmit).await?;

        self.upstream.read_buf(&mut buf).await?;

        let req = ConnectRequest::decode(&mut buf)?.unwrap();

        info!(target = %req.target, "Try to connect to {}", req.target);
        let mut socks5 = match self.set_downstream(req.target.clone()).await {
            Ok(()) => {
                let (transmit, socks5) = socks5?.connect(req, ConnectedStatus::Succeeded);
                self.send_transmit(transmit).await?;
                Ok(socks5?)
            }
            Err(e) => {
                let target = req.target.clone();
                let (transmit, _) = socks5?.connect(req, ConnectedStatus::HostUnreachable);

                warn!(target = %target, "Connect to {} failed: {}", target, e);

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

                info!(
                    ip = %ip.ip(),
                    port = %ip.port(),
                    "Connected to remote endpoint via tcp"
                );

                Stream::Tcp(stream)
            }
            Address::Domain(domain, port) => match NodeId::from_str(domain) {
                Ok(node_id) => {
                    let conn = self.endpoint.connect(node_id.0, ALPN).await?;

                    let mut stream = conn.open_bi().await?;

                    let (mut data, _) = wave_core::Connection::connect(domain, *port)?;

                    stream.0.write_all_buf(&mut data).await?;

                    info!(%node_id, "Connected to remote endpoint via iroh");

                    Stream::Iroh(stream.0, stream.1)
                }
                Err(_e) => {
                    let stream = TcpStream::connect((domain.as_ref(), *port)).await?;

                    info!(%domain, %port, "Connected to remote endpoint via tcp");

                    Stream::Tcp(stream)
                }
            },
        };
        self.downstream = Some((addr, stream));
        Ok(())
    }

    async fn get_stream(&mut self, address: &Address) -> anyhow::Result<&mut Stream> {
        if Address::Ip(self.upstream_address) == *address {
            return Ok(&mut self.upstream);
        }

        if let Some((addr, stream)) = self.downstream.as_mut() {
            if addr == address {
                return Ok(stream);
            }
        }
        Err(anyhow::anyhow!("downstream address mismatch"))
    }

    async fn send_transmit(
        &mut self,
        Transmit { to, mut data, .. }: Transmit,
    ) -> anyhow::Result<()> {
        let stream = self.get_stream(&to).await?;

        stream.write_all_buf(&mut data).await?;

        debug!(address = %to, data_size = data.len(), "Send data to remote endpoint");

        Ok(())
    }
}
