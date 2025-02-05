use crate::Stream;
use anyhow::anyhow;
use iroh::{endpoint::Incoming, Endpoint};
use std::net::SocketAddr;
use tracing::info;

pub struct Server {
    endpoint: Endpoint,
    downstream_peer: SocketAddr,
}

impl Server {
    pub fn new(endpoint: Endpoint, downstream_peer: SocketAddr) -> Self {
        Self {
            endpoint,
            downstream_peer,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        loop {
            let conn = self
                .endpoint
                .accept()
                .await
                .ok_or_else(|| anyhow!("Accept connection failed"))?;

            tokio::spawn(async move {
                Self::handle(conn, self.downstream_peer)
                    .await
                    .inspect_err(|e| {
                        tracing::error!("handle incomming error: {}", e);
                    })
                    .expect("handle incomming failed");
            });
        }
    }

    async fn handle(incoming: Incoming, downstream_peer: SocketAddr) -> anyhow::Result<()> {
        let remote_addr = incoming.remote_address();
        let local_addr = incoming.local_ip();
        let (send_stream, recv_stream) = incoming.await?.accept_bi().await?;

        info!(local = ?local_addr, remote = %remote_addr, "Accept connection");

        let mut downstream = tokio::net::TcpStream::connect(downstream_peer).await?;

        info!("proxy to {}", downstream_peer);

        let mut upstream = Stream::Iroh(send_stream, recv_stream);

        tokio::io::copy_bidirectional(&mut upstream, &mut downstream).await?;
        Ok(())
    }
}
