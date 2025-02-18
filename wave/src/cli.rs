use crate::{client::Client, config, server::ServerService, store::Store, ALPN};
use clap::Parser;
use iroh::Endpoint;
use tracing::info;
use wave_core::{router::Router, NodeId, Server};
use wave_proxy::Proxy;

#[derive(Parser)]
pub enum Cli {
    Run,
}

pub async fn run_cli() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let dir = dirs::home_dir()
        .map(|d| d.join(".wave"))
        .ok_or(anyhow::anyhow!("Can't find home dir"))?;
    info!("Find store dir: {}", dir.display());
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    let store = Store::from_path(dir.join("store"))?;

    let secret = store.get_secret_key()?;
    let config = if let Some(config) = config::init_config()? {
        store.put_config(&config)?;
        config
    } else {
        match store.get_config()? {
            Some(config) => config,
            None => {
                let config = config::Config::default();
                store.put_config(&config)?;
                config
            }
        }
    };
    match cli {
        Cli::Run => {
            let server = {
                let mut builder = Router::builder();
                builder = builder.add("".parse()?, "127.0.0.1".parse()?);
                for (k, v) in config.router {
                    builder = builder.add(k.parse()?, v.parse()?);
                }

                Server::new(builder.build()?)
            };

            let proxy = Proxy::new(config.proxy.socks5.parse()?);

            let mut ep = Endpoint::builder()
                .alpns(vec![ALPN.into()])
                .discovery_local_network()
                .discovery_n0()
                // .discovery_dht()
                .bind_addr_v4(config.server.address.parse()?);

            if let Some(secret) = secret {
                ep = ep.secret_key(secret);
            } else {
                let secret = iroh::SecretKey::generate(&mut rand::rngs::OsRng);
                store.put_secret_key(&secret)?;
                ep = ep.secret_key(secret);
            };

            let ep = ep.bind().await?;

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
