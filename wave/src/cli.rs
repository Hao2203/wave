use crate::{client::Client, server::Server, ALPN};
use clap::{Args, Parser};
use iroh::Endpoint;
use std::net::SocketAddr;
use tracing::info;

const SERVER_ENDPOINT: &str = "127.0.0.1:8282";

const CLIENT_ENDPOINT: &str = "127.0.0.1:8181";

const CLIENT_PROXY: &str = "127.0.0.1:8182";

const DOWNSTREAM: &str = "127.0.0.1:8284";

#[derive(Parser)]
pub enum Cli {
    Bind(BindArgs),
}

#[derive(Args)]
pub struct BindArgs {
    pub addr: Option<String>,
}

pub async fn run_cli() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli {
        Cli::Bind(args) => {
            let addr = args.addr.unwrap_or_else(|| DOWNSTREAM.to_string());
            spawn_client();
            spawn_server(addr.parse()?).await;
        }
    }

    Ok(())
}

fn spawn_client() {
    tokio::spawn(async move {
        info!("start client");
        let client = Client::new(
            CLIENT_PROXY,
            Endpoint::builder()
                .discovery_local_network()
                .discovery_n0()
                .discovery_dht()
                .bind_addr_v4(CLIENT_ENDPOINT.parse().unwrap())
                .bind()
                .await
                .unwrap(),
        )
        .await
        .unwrap();

        client.run().await.unwrap();
    });
}

async fn spawn_server(bind: SocketAddr) {
    info!("start server");
    let ep = Endpoint::builder()
        .alpns(vec![ALPN.into()])
        .discovery_local_network()
        .discovery_n0()
        .discovery_dht()
        .bind_addr_v4(SERVER_ENDPOINT.parse().unwrap())
        .bind()
        .await
        .unwrap();
    let node_id = ep.node_id();

    println!("node_id: {}, bind: {}", node_id, bind);

    let server = Server::new(ep, bind);

    server.run().await.unwrap();
}
