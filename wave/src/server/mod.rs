use crate::Stream;
use bytes::BytesMut;
use futures_lite::FutureExt;
use iroh::{endpoint::Incoming, Endpoint};
use std::sync::Arc;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};
use tracing::info;
use wave_core::{server::Host, Connection, NodeId, Server, WavePacket};

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
            tokio::select! {
                Some(conn) = self.endpoint.accept() => {
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
        }
    }

    async fn handle(incoming: Incoming, server: Arc<Server>) -> anyhow::Result<()> {
        let iroh_conn = incoming.await?;
        let (mut send_stream, mut recv_stream) = iroh_conn.accept_bi().await?;

        let mut upstream_buf = BytesMut::with_capacity(1024);
        let wave_packet = loop {
            recv_stream.read_buf(&mut upstream_buf).await?;
            if let Some(wave_packet) = WavePacket::decode(&mut upstream_buf)? {
                break wave_packet;
            }
        };
        let remote_node_id = iroh_conn.remote_node_id()?;
        let (conn, host) = server.accept(NodeId(remote_node_id), wave_packet);

        let host = match host {
            Ok(host) => host,
            Err(fallback) => {
                send_stream.write_all_buf(&mut fallback.bytes()).await?;
                return Ok(());
            }
        };

        Self::handle_stream(
            Stream::Iroh(send_stream, recv_stream),
            upstream_buf,
            conn,
            host,
        )
        .await?;

        Ok(())
    }

    async fn handle_stream<S>(
        mut upstream: S,
        mut upstream_buf: BytesMut,
        conn: Connection,
        target: Host,
    ) -> anyhow::Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let downstream_host = target;

        let mut downstream = match &downstream_host {
            Host::Ip(ip) => TcpStream::connect((*ip, conn.port())).await?,
            Host::Domain(domain) => TcpStream::connect((domain.as_ref(), conn.port())).await?,
        };

        info!("proxy to {}:{}", downstream_host, conn.port());

        upstream_buf.reserve(1024);
        let mut downstream_buf = BytesMut::with_capacity(1024);

        loop {
            if !downstream_buf.is_empty() {
                upstream.write_all_buf(&mut downstream_buf).await?;
            }
            if !upstream_buf.is_empty() {
                downstream.write_all_buf(&mut upstream_buf).await?;
            }
            upstream
                .read_buf(&mut upstream_buf)
                .race(downstream.read_buf(&mut downstream_buf))
                .await?;
        }
    }
}

// pub struct SelfConnecting {
//     pub upstream: Stream,
//     pub conn: Connection,
// }
