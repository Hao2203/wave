// #![allow(unused)]
use super::*;
use bytes::BytesMut;
use fast_socks5::{client::*, Socks5Command};
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::TcpListener,
};

#[tokio::test]
async fn test() {
    use crate::socks5::*;

    let server_task = tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:1234").await.unwrap();
        let (stream, addr) = listener.accept().await.unwrap();
        let (info, mut tunnel) = Socks5 {}.serve(Box::pin(stream), addr).await.unwrap();
        assert_eq!(info.target, Target::Ip("127.0.0.1:80".parse().unwrap()));

        let data = test_data();
        let mut buf = BytesMut::with_capacity(data.len());
        tunnel.read_buf(&mut buf).await.unwrap();
        assert_eq!(buf.to_vec(), data);
    });

    let client_task = tokio::spawn(async move {
        let mut client = Socks5Stream::connect_raw(
            Socks5Command::TCPConnect,
            "127.0.0.1:1234",
            "127.0.0.1".into(),
            80,
            None,
            Default::default(),
        )
        .await
        .unwrap();
        client.write_all(&test_data()).await.unwrap();
    });

    server_task.await.unwrap();
    client_task.await.unwrap();
}

fn test_data() -> Vec<u8> {
    let domain = "www.example.com".to_string();
    // construct our request, with a dynamic domain
    let mut headers = vec![];
    headers.extend_from_slice("GET / HTTP/1.1\r\nHost: ".as_bytes());
    headers.extend_from_slice(domain.as_bytes());
    headers
        .extend_from_slice("\r\nUser-Agent: fast-socks5/0.1.0\r\nAccept: */*\r\n\r\n".as_bytes());

    headers
}
