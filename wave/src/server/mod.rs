use crate::Stream;
use bytes::BytesMut;
use futures_lite::FutureExt;
use iroh::{endpoint::Incoming, Endpoint};
use std::{net::SocketAddr, sync::Arc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::info;
use wave_core::{NodeId, Server, WavePacket};

pub struct ServerService {
    server: Arc<wave_core::Server>,
    endpoint: Endpoint,
}

impl ServerService {
    pub fn new(server: Arc<wave_core::Server>, endpoint: Endpoint) -> Self {
        Self { server, endpoint }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        loop {
            let conn = if let Some(conn) = self.endpoint.accept().await {
                conn
            } else {
                continue;
            };

            let server = self.server.clone();
            tokio::spawn(async move {
                Self::handle(conn, server)
                    .await
                    .inspect_err(|e| {
                        tracing::error!("handle incomming error: {}", e);
                    })
                    .expect("handle incomming failed");
            });
        }
    }

    async fn handle(incoming: Incoming, server: Arc<Server>) -> anyhow::Result<()> {
        let remote_addr = incoming.remote_address();
        let local_addr = incoming.local_ip();
        let iroh_conn = incoming.await?;
        let (send_stream, mut recv_stream) = iroh_conn.accept_bi().await?;

        let mut downstream_buf = BytesMut::with_capacity(1024);
        let wave_packet = loop {
            recv_stream.read_buf(&mut downstream_buf).await?;
            if let Some(wave_packet) = WavePacket::decode(&mut downstream_buf)? {
                break wave_packet;
            }
        };
        let remote_node_id = iroh_conn.remote_node_id()?;
        let (conn, ip) = server.accept(NodeId(remote_node_id), wave_packet);

        let ip = match ip {
            Some(ip) => ip,
            None => {
                return Err(anyhow::anyhow!("no ip for subdomain {}", conn.subdomain()));
            }
        };

        let downstream_peer = SocketAddr::new(ip, conn.port());

        info!(local = ?local_addr, remote = %remote_addr, "Accept connection");

        let mut downstream = tokio::net::TcpStream::connect(downstream_peer).await?;

        info!("proxy to {}", downstream_peer);

        let mut upstream = Stream::Iroh(send_stream, recv_stream);

        let mut upstream_buf = BytesMut::with_capacity(1024);
        loop {
            upstream
                .read_buf(&mut upstream_buf)
                .race(downstream.read_buf(&mut downstream_buf))
                .await?;
            if !upstream_buf.is_empty() {
                downstream.write_all_buf(&mut upstream_buf).await?;
            }
            if !downstream_buf.is_empty() {
                upstream.write_all_buf(&mut downstream_buf).await?;
            }
        }

        // tokio::io::copy_bidirectional(&mut upstream, &mut downstream).await?;
    }
}
