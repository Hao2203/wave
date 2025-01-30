use std::net::SocketAddr;

use tokio::io::AsyncReadExt;

pub struct Client {
    local: SocketAddr,
}

impl Client {
    pub async fn run(self) -> anyhow::Result<()> {
        todo!()
    }
}
