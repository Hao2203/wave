use bytes::BytesMut;
use futures_lite::FutureExt;
use iroh::Endpoint;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};
use tracing::{info, warn};
use ulid::Ulid;
use wave_core::{Connection, Host, NodeId, Server, WavePacket};

pub struct ServerService {
    server: wave_core::Server,
    endpoint: Endpoint,
}

impl ServerService {
    pub fn new(server: wave_core::Server, endpoint: Endpoint) -> Self {
        Self { server, endpoint }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        loop {
            if let Some(conn) = self.endpoint.accept().await {
                let server = self.server.clone();
                tokio::spawn(async move {
                    let conn = match conn.await {
                        Ok(conn) => conn,
                        Err(e) => {
                            tracing::error!("accept connection error when handshake: {}", e);
                            return;
                        }
                    };
                    if let Err(_e) = Self::server_handle(conn, server).await {
                        // tracing::error!("handle connection error: {}", e);
                    }
                });
            }
        }
    }

    #[tracing::instrument(skip_all, ret, err, fields(server_handle_id = Ulid::new().to_string(), remote_node_id))]
    async fn server_handle(conn: iroh::endpoint::Connection, server: Server) -> anyhow::Result<()> {
        let remote_node_id = NodeId(conn.remote_node_id()?);

        tracing::Span::current().record("remote_node_id", remote_node_id.to_string());

        let (send_stream, mut recv_stream) = conn.accept_bi().await?;

        let mut upstream_buf = BytesMut::with_capacity(1024);
        let wave_packet = loop {
            upstream_buf.reserve(1024);
            recv_stream.read_buf(&mut upstream_buf).await?;
            if let Some(wave_packet) = WavePacket::decode(&mut upstream_buf)? {
                break wave_packet;
            }
        };

        let (conn, host) = server.accept(remote_node_id, wave_packet);

        let host = match host {
            Some(host) => host,
            None => {
                warn!(
                    node_id = ?remote_node_id,
                    subdomain = ?conn.subdomain(),
                    "No host found for subdomain"
                );
                return Err(anyhow::anyhow!("No host found for subdomain"));
            }
        };

        Self::handle_stream(
            tokio::io::join(recv_stream, send_stream),
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
