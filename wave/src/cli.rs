use crate::{client::Client, config, server::ServerService, ALPN};
use clap::{Args, Parser};
use iroh::Endpoint;
use std::sync::Arc;
use tracing::info;
use wave_core::{NodeId, Server};

const SERVER_ENDPOINT: &str = "127.0.0.1:8282";

const CLIENT_PROXY: &str = "127.0.0.1:8182";

const DOWNSTREAM: &str = "127.0.0.1";

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
    let config = config::init_config()?;
    match cli {
        Cli::Bind(args) => {
            let addr = args.addr.unwrap_or_else(|| DOWNSTREAM.to_string());
            let mut server = Server::try_from_iter(config.router)?;
            server.add("".parse()?, addr.parse()?);

            let ep = Endpoint::builder()
                .alpns(vec![ALPN.into()])
                .discovery_local_network()
                .discovery_n0()
                .discovery_dht()
                .bind_addr_v4(SERVER_ENDPOINT.parse().unwrap())
                .bind()
                .await
                .unwrap();

            let server = Arc::new(server);
            spawn_client(ep.clone(), server.clone());
            spawn_server(ep, server).await;
        }
    }

    Ok(())
}

fn spawn_client(ep: Endpoint, server: Arc<Server>) {
    tokio::spawn(async move {
        info!("start client");
        let client = Client::new(server, CLIENT_PROXY, ep).await.unwrap();

        client.run().await.unwrap();
    });
}

async fn spawn_server(ep: Endpoint, server: Arc<Server>) {
    info!("start server");
    let node_id = NodeId(ep.node_id());

    println!("node_id: {}", node_id);

    let server = ServerService::new(server, ep);

    server.run().await.unwrap();
}
