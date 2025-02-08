use crate::Stream;
use bytes::BytesMut;
use futures_lite::FutureExt;
use iroh::{endpoint::Incoming, Endpoint};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::info;
use wave_core::{server::Host, NodeId, Server, WavePacket};

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
        let (mut send_stream, mut recv_stream) = iroh_conn.accept_bi().await?;

        let mut downstream_buf = BytesMut::with_capacity(1024);
        let wave_packet = loop {
            recv_stream.read_buf(&mut downstream_buf).await?;
            if let Some(wave_packet) = WavePacket::decode(&mut downstream_buf)? {
                break wave_packet;
            }
        };
        let remote_node_id = iroh_conn.remote_node_id()?;
        let (conn, host) = server.accept(NodeId(remote_node_id), wave_packet);

        let downstream_host = match host {
            Ok(ip) => ip,
            Err(fallback) => {
                send_stream.write_all_buf(&mut fallback.bytes()).await?;
                return Err(anyhow::anyhow!("no ip for subdomain {}", conn.subdomain()));
            }
        };

        info!(local = ?local_addr, remote = %remote_addr, "Accept connection");

        let mut downstream = match &downstream_host {
            Host::Ip(ip) => TcpStream::connect((*ip, conn.port())).await?,
            Host::Domain(domain) => TcpStream::connect((domain.as_ref(), conn.port())).await?,
        };

        info!("proxy to {}", downstream_host);

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
