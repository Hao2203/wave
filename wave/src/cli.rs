use crate::{client::Client, config, server::ServerService, ALPN};
use clap::{Args, Parser};
use iroh::Endpoint;
use tracing::info;
use wave_core::{router::Router, NodeId, Server};
use wave_proxy::Proxy;

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
                    builder = builder.add("".parse()?, "127.0.0.1".parse()?);
                }
                Server::new(builder.build()?)
            };

            let proxy = Proxy::new(config.proxy.socks5.parse()?);

            let ep = Endpoint::builder()
                .alpns(vec![ALPN.into()])
                .discovery_local_network()
                .discovery_n0()
                // .discovery_dht()
                .bind_addr_v4(config.server.address.parse()?)
                .bind()
                .await?;

            spawn_client(ep.clone(), server.clone(), proxy);
            spawn_server(ep, server).await;
        }
    }

    Ok(())
}

fn spawn_client(ep: Endpoint, server: Server, proxy: Proxy) {
    tokio::spawn(async move {
        info!("start client");
        let client = Client::new(proxy, server, ep);

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
