#![allow(unused)]
use super::*;

#[tokio::test]
async fn test() {
    use crate::socks5::*;

    let proxy = ProxyServer::builder(Socks5 {})
        .bind("127.0.0.1:1234".parse().unwrap())
        .build()
        .await
        .unwrap();

    while let Ok(res) = proxy.accept().await {
        tokio::spawn(async move {
            let res = res.start_proxy().await.unwrap();
            println!("{:?}", res.target);
        });
    }
}
