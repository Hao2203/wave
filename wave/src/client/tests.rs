use iroh::Endpoint;
use reqwest::Proxy;
use tokio::net::TcpListener;

use super::Client;

#[tokio::test]
async fn test() {
    tracing_subscriber::fmt().init();

    let proxy = Proxy::http("socks5://127.0.0.1:8181").unwrap();
    let http_client = reqwest::Client::builder().proxy(proxy).build().unwrap();
    let client = Client {
        listener: TcpListener::bind("127.0.0.1:8181").await.unwrap(),
        endpoint: Endpoint::builder().bind().await.unwrap(),
    };
    let task = tokio::spawn(client.run());
    let text = http_client
        .get("https://www.baidu.com")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("text: {}", text);
    task.await.unwrap().unwrap();
}

// #[tokio::test]
// async fn test2() {
//     tracing_subscriber::fmt().init();

//     let proxy = Proxy::http("socks5://127.0.0.1:8181").unwrap();
//     let http_client = reqwest::Client::builder().proxy(proxy).build().unwrap();
//     let client = Client {
//         listener: TcpListener::bind("127.0.0.1:8181").await.unwrap(),
//         endpoint: Endpoint::builder().bind().await.unwrap(),
//     };
//     client.run().await.unwrap();
// }
