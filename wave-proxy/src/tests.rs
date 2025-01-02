// #![allow(unused)]
use super::*;
use async_trait::async_trait;
use bytes::BytesMut;
use fast_socks5::{client::*, Socks5Command};
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::TcpListener,
};

#[tokio::test]
async fn test() {
    use crate::socks5::*;

    struct Ctx(SocketAddr, Vec<u8>);

    #[async_trait]
    impl ProxyCtx for Ctx {
        fn local_addr(&self) -> SocketAddr {
            self.0
        }
        async fn proxy_info_filter(&mut self, info: &ProxyInfo) -> Result<()> {
            let target = &info.target;
            assert_eq!(*target, Target::Ip("127.0.0.1:80".parse().unwrap()));
            Ok(())
        }
        async fn process_tunnel(&mut self, tunnel: &mut (dyn Connection + Unpin)) -> Result<()> {
            let mut buf = BytesMut::with_capacity(self.1.len());
            tunnel.read_buf(&mut buf).await.unwrap();
            assert_eq!(buf.to_vec(), self.1);
            Ok(())
        }
    }

    let server_task = tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:1234").await.unwrap();
        let incoming = listener.accept().await.unwrap();
        let stream = incoming.0;
        let ctx = Ctx(incoming.1, data());
        let builder = Builder::new(Socks5 {});
        builder.serve(ctx, stream).await.unwrap();
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
        client.write_all(&data()).await.unwrap();
    });

    server_task.await.unwrap();
    client_task.await.unwrap();
}

fn data() -> Vec<u8> {
    let domain = "www.example.com".to_string();
    // construct our request, with a dynamic domain
    let mut headers = vec![];
    headers.extend_from_slice("GET / HTTP/1.1\r\nHost: ".as_bytes());
    headers.extend_from_slice(domain.as_bytes());
    headers
        .extend_from_slice("\r\nUser-Agent: fast-socks5/0.1.0\r\nAccept: */*\r\n\r\n".as_bytes());

    headers
}
