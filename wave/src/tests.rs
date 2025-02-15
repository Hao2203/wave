use std::net::SocketAddr;

use crate::{client::Client, server::ServerService, ALPN};
use iroh::Endpoint;
use reqwest::{Proxy, Response};
use tracing::info;
use wave_core::{router::Router, NodeId, Server};

// const SERVER_ENDPOINT: &str = "127.0.0.1:8282";

const CLIENT_PROXY: &str = "127.0.0.1:8182";

#[tokio::test]
async fn test() {
    tracing_subscriber::fmt().init();

    let http_server_addr = spawn_http_server().await.unwrap();

    let router = Router::builder()
        .add("".parse().unwrap(), http_server_addr.ip().into())
        .build();

    let node_id = spawn_wave(router).await.unwrap();

    // let node_id = spawn_wave(Router::default()).await.unwrap();
    let res = get_http_response(node_id, http_server_addr.port())
        .await
        .unwrap();

    assert_eq!(res.text().await.unwrap(), "hello world");
}

async fn spawn_wave(router: Router) -> anyhow::Result<NodeId> {
    let ep = Endpoint::builder()
        .alpns(vec![ALPN.into()])
        .discovery_local_network()
        // .bind_addr_v4(SERVER_ENDPOINT.parse().unwrap())
        .bind()
        .await?;
    let node_id = NodeId(ep.node_id());

    info!("node_id: {}", node_id);

    let server = Server::new(router);
    let server_service = ServerService::new(server.clone(), ep.clone());

    tokio::spawn(async move {
        info!("start server");

        server_service.run().await.unwrap();
    });

    let client_server = Client::new(CLIENT_PROXY, ep, server).await.unwrap();
    tokio::spawn(async move {
        info!("start client");

        client_server.run().await.unwrap();
    });

    Ok(node_id)
}

async fn spawn_http_server() -> anyhow::Result<SocketAddr> {
    let router = axum::Router::new().route("/", axum::routing::get(|| async { "hello world" }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let socket = listener.local_addr()?;
    tokio::spawn(async move { axum::serve(listener, router).await });
    Ok(socket)
}

async fn get_http_response(node_id: NodeId, http_server_port: u16) -> anyhow::Result<Response> {
    let proxy = Proxy::all(format!("socks5h://{}", CLIENT_PROXY)).unwrap();
    let http_client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .unwrap();

    println!("node_id: {}", node_id);
    let res = http_client
        .get(format!("http://{}:{}", node_id, http_server_port))
        .send()
        .await?;
    Ok(res)
}
