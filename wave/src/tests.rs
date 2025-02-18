use crate::{client::Client, server::ServerService, ALPN};
use iroh::Endpoint;
use reqwest::{Proxy, Response};
use std::net::SocketAddr;
use tracing::info;
use wave_core::{router::Router, NodeId, Server};
use wave_proxy::Proxy as WaveProxy;

#[tokio::test]
async fn test() {
    tracing_subscriber::fmt().init();

    let http_server_addr = spawn_http_server().await.unwrap();

    let router = Router::builder()
        .add("".parse().unwrap(), http_server_addr.ip().into())
        .build()
        .unwrap();

    let proxy = WaveProxy::new("127.0.0.1:8182".parse().unwrap());
    let socks5_addr = proxy.socks5_addr();
    let node_id = spawn_wave(router.clone(), proxy).await.unwrap();

    let res = get_http_response(socks5_addr, node_id, http_server_addr.port())
        .await
        .unwrap();

    assert_eq!(res.text().await.unwrap(), "hello world");

    let proxy = WaveProxy::new("127.0.0.1:8183".parse().unwrap());
    let socks5_addr = proxy.socks5_addr();
    let _ = spawn_wave(router, proxy).await.unwrap();
    let res = get_http_response(socks5_addr, node_id, http_server_addr.port())
        .await
        .unwrap();

    assert_eq!(res.text().await.unwrap(), "hello world");
}

async fn spawn_wave(router: Router, proxy: WaveProxy) -> anyhow::Result<NodeId> {
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

        server_service.run().await.expect("run server failed");
    });

    let client_server = Client::new(proxy, server, ep);
    tokio::spawn(async move {
        info!("start client");

        client_server.run().await.expect("run client failed");
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

async fn get_http_response(
    socks5_addr: SocketAddr,
    node_id: NodeId,
    http_server_port: u16,
) -> anyhow::Result<Response> {
    info!("socks5_addr: {}", socks5_addr);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let proxy = Proxy::all(format!("socks5h://{}", socks5_addr))?;
    let http_client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(std::time::Duration::from_secs(20))
        .build()?;

    let res = http_client
        .get(format!("http://{}:{}", node_id, http_server_port))
        .send()
        .await?;
    Ok(res)
}
