use crate::{client::Client, config, server::ServerService, ALPN};
use clap::{Args, Parser};
use iroh::Endpoint;
use tracing::info;
use wave_core::{router::Router, NodeId, Server};

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
            let server = {
                let mut builder = Router::builder();
                for (k, v) in config.router {
                    builder = builder.add(k.parse()?, v.parse()?);
                }
                if let Some(addr) = args.addr {
                    builder = builder.add("".parse()?, addr.parse()?);
                } else {
                    builder = builder.add("".parse()?, DOWNSTREAM.parse()?);
                }
                Server::new(builder.build())
            };

            let ep = Endpoint::builder()
                .alpns(vec![ALPN.into()])
                .discovery_local_network()
                .discovery_n0()
                // .discovery_dht()
                .bind_addr_v4(SERVER_ENDPOINT.parse().unwrap())
                .bind()
                .await?;

            spawn_client(ep.clone(), server.clone());
            spawn_server(ep, server).await;
        }
    }

    Ok(())
}

fn spawn_client(ep: Endpoint, server: Server) {
    tokio::spawn(async move {
        info!("start client");
        let client = Client::new(CLIENT_PROXY, ep, server).await.unwrap();

        client.run().await.unwrap();
    });
}

async fn spawn_server(ep: Endpoint, server: Server) {
    info!("start server");
    let node_id = NodeId(ep.node_id());

    println!("node_id: {}", node_id);

    let server = ServerService::new(server, ep);

    server.run().await.unwrap();
}
