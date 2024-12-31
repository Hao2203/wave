#![allow(unused)]
use super::*;
use fast_socks5::{client::*, Socks5Command};
use futures_lite::future::zip;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

#[tokio::test]
async fn test() {
    use crate::socks5::*;

    let server = ProxyServer::builder(Socks5 {})
        .bind("127.0.0.1:1234".parse().unwrap())
        .build()
        .await
        .unwrap();

    let domain = "www.example.com".to_string();
    // construct our request, with a dynamic domain
    let mut headers = vec![];
    headers.extend_from_slice("GET / HTTP/1.1\r\nHost: ".as_bytes());
    headers.extend_from_slice(domain.as_bytes());
    headers
        .extend_from_slice("\r\nUser-Agent: fast-socks5/0.1.0\r\nAccept: */*\r\n\r\n".as_bytes());

    // flush headers
    let headers_clone = headers.clone();
    let client_task = tokio::spawn(async move {
        let mut config = Config::default();
        let mut client = Socks5Stream::connect_raw(
            Socks5Command::TCPConnect,
            "127.0.0.1:1234",
            "127.0.0.1".into(),
            80,
            None,
            config,
        )
        .await
        .unwrap();
        client.write_all(&headers_clone).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    });

    if let Ok(res) = server.accept().await {
        let server_task = tokio::spawn(async move {
            let mut res = res.start_proxy().await.unwrap();
            assert_eq!(res.target, Target::Ip("127.0.0.1:80".parse().unwrap()));

            let mut result = headers.clone();

            res.io.read_exact(&mut result).await.unwrap();

            assert_eq!(headers, result);
        });
        zip(client_task, server_task).await;
    }
}
