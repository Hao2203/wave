use std::net::SocketAddr;

pub struct Client {
    addrs: Vec<SocketAddr>,
}

impl Client {
    pub async fn run(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
