use std::net::SocketAddr;

use tokio::io::AsyncReadExt;
use wave_proxy::protocol::socks5::{Output, Socks5};

pub struct Client {
    local: SocketAddr,
}

impl Client {
    pub async fn run(self) -> anyhow::Result<()> {
        let ep = iroh::Endpoint::builder().bind().await?;
        let listener = tokio::net::TcpListener::bind(self.local).await?;

        loop {
            let (socket, local) = listener.accept().await?;
            let socks5: Socks5 = Socks5::new(local);

            tokio::spawn(async move {
                let _ = Self::process(socket, socks5).await;
            });
        }
    }

    async fn process(mut socket: tokio::net::TcpStream, mut socks5: Socks5) -> anyhow::Result<()> {
        loop {
            match socks5.poll_output()? {
                Output::Pending => {
                    // let data = socket.read_exact(buf)
                }
                Output::Handshake(transmit) => {
                    todo!()
                }
                Output::TcpConnect(mut connect) => {
                    todo!()
                }
                Output::Relay(transmit) => {
                    break;
                }
            }
        }
        todo!()
    }
}
