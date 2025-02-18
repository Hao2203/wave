// #![allow(unused)]
use crate::{Stream, ALPN};
use bytes::BytesMut;
use futures_lite::FutureExt;
use iroh::Endpoint;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, info};
use ulid::Ulid;
use wave_core::{server::Fallback, Connection, Host, Server};
use wave_proxy::{
    protocol::socks5::{
        types::{ConnectRequest, ConnectedStatus, HandshakeRequest},
        NoAuthHandshake,
    },
    Address, Proxy, Transmit,
};

#[cfg(test)]
mod tests;

pub struct Client {
    proxy: Proxy,
    endpoint: Endpoint,
    server: Server,
}

impl Client {
    pub fn new(proxy: Proxy, server: Server, endpoint: Endpoint) -> Self {
        Self {
            proxy,
            endpoint,
            server,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.proxy.socks5_addr()).await?;
        loop {
            let (stream, local) = listener.accept().await?;
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
                if let Err(_e) = handler.client_handle().await {
                    // tracing::error!("handle incomming error: {}", e);
                }
            });
        }
    }
}

#[derive(Debug)]
struct Handler {
    server: Server,
    local: SocketAddr,
    upstream_address: SocketAddr,
    endpoint: Endpoint,
    upstream: Stream,
    downstream: Option<(Address, Stream)>,
}

impl Handler {
    #[tracing::instrument(skip_all,ret, err, fields(client_handle_id = Ulid::new().to_string()))]
    async fn client_handle(mut self) -> anyhow::Result<()> {
        info!("Connect from {}", self.upstream_address);

        let socks5 = NoAuthHandshake::new(self.local, self.upstream_address);

        let mut buf = BytesMut::with_capacity(1024);
        self.upstream.read_buf(&mut buf).await?;
        let req = HandshakeRequest::decode(&mut buf)?.unwrap();
        let (transmit, socks5) = socks5.handshake(req);
        self.send_transmit(transmit).await?;

        self.upstream.read_buf(&mut buf).await?;

        let req = ConnectRequest::decode(&mut buf)?.unwrap();

        info!(target = %req.target, "Try to connect " );
        let (transmit, socks5) = socks5?.connect(req.clone(), ConnectedStatus::Succeeded);
        self.send_transmit(transmit).await?;

        let mut socks5 = match self.connect_to_downstream(req.target.clone()).await {
            Ok(stream) => {
                self.downstream = Some((req.target.clone(), stream));
                Ok(socks5?)
            }
            Err(e) => {
                let fallback = Fallback::default();
                self.upstream.write_all_buf(&mut fallback.bytes()).await?;
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

    async fn connect_to_downstream(&self, addr: Address) -> anyhow::Result<Stream> {
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
            Address::Domain(domain, port) => match Connection::connect(domain, *port) {
                Ok((mut data, conn)) => {
                    let node_id = conn.node_id();
                    if node_id.0 == self.endpoint.node_id() {
                        info!(?node_id, "Connected to self");

                        let target = self.server.get_target(&conn.subdomain());
                        let res = match target {
                            Some(Host::Ip(ip)) => {
                                info!(%ip, %port, "Self connected, route to target via tcp");

                                let stream = TcpStream::connect((ip, *port)).await?;
                                Ok(Stream::Tcp(stream))
                            }
                            Some(Host::Domain(domain)) => {
                                info!(%domain, %port, "Self connected, route to target via tcp");

                                let stream = TcpStream::connect((domain.as_ref(), *port)).await?;
                                Ok(Stream::Tcp(stream))
                            }
                            None => Err(anyhow::anyhow!(
                                "no target for subdomain {}",
                                conn.subdomain()
                            )),
                        };
                        return res;
                    }

                    let conn = self.endpoint.connect(node_id.0, ALPN).await?;

                    let mut stream = conn.open_bi().await?;

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
        Ok(stream)
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
