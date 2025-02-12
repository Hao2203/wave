use crate::{client::Client, server::ServerService, ALPN};
use iroh::Endpoint;
use reqwest::Proxy;
use std::sync::Arc;
use tracing::info;
use wave_core::{NodeId, Server};

const SERVER_ENDPOINT: &str = "127.0.0.1:8282";

const CLIENT_ENDPOINT: &str = "127.0.0.1:8181";

const CLIENT_PROXY: &str = "127.0.0.1:8182";

const DOWNSTREAM: &str = "127.0.0.1";

#[tokio::test]
async fn test() {
    tracing_subscriber::fmt().init();

    let (tx, rx) = tokio::sync::oneshot::channel();

    let mut server = Server::default();
    server.add("".parse().unwrap(), DOWNSTREAM.parse().unwrap());
    let server = Arc::new(server);

    let server_clone = server.clone();
    tokio::spawn(async move {
        info!("start server");
        let ep = Endpoint::builder()
            .alpns(vec![ALPN.into()])
            .discovery_local_network()
            .bind_addr_v4(SERVER_ENDPOINT.parse().unwrap())
            .bind()
            .await
            .unwrap();
        let node_id = NodeId(ep.node_id());
        tx.send(node_id).unwrap();

        info!("node_id: {}", node_id);

        let server = ServerService::new(server_clone, ep);

        server.run().await.unwrap();
    });

    tokio::spawn(async move {
        info!("start client");
        let client = Client::new(
            CLIENT_PROXY,
            Endpoint::builder()
                .discovery_local_network()
                .bind_addr_v4(CLIENT_ENDPOINT.parse().unwrap())
                .bind()
                .await
                .unwrap(),
            server,
        )
        .await
        .unwrap();

        client.run().await.unwrap();
    });

    tokio::spawn(async move { http_app().await.unwrap() });

    let res = http_client(rx.await.unwrap()).await.unwrap();
    assert_eq!(res, "hello world");
}

async fn http_app() -> anyhow::Result<()> {
    let router = axum::Router::new().route("/", axum::routing::get(|| async { "hello world" }));
    let listener = tokio::net::TcpListener::bind((DOWNSTREAM, 8183))
        .await
        .unwrap();
    Ok(axum::serve(listener, router).await?)
}

async fn http_client(node_id: NodeId) -> anyhow::Result<String> {
    let proxy = Proxy::all(format!("socks5h://{}", CLIENT_PROXY)).unwrap();
    let http_client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .unwrap();

    println!("node_id: {}", node_id);
    let res = http_client
        .get(format!("http://{}:8183", node_id))
        .send()
        .await?
        .text()
        .await?;
    Ok(res)
}
