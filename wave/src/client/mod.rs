use std::net::SocketAddr;

use tokio::io::AsyncReadExt;
use wave_proxy::protocol::socks5::{Output, Socks5};

pub struct Client {
    local: SocketAddr,
}

impl Client {
    pub async fn run(self) -> anyhow::Result<()> {
        todo!()
    }
}
