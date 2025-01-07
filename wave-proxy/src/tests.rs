// #![allow(unused)]
use super::*;
use fast_socks5::{client::*, Socks5Command};
use server::ProxyServer;
use tokio::{io::AsyncWriteExt as _, net::TcpListener};

#[tokio::test]
async fn test() {
    use crate::socks5::*;

    pub struct TestApp;
    impl ProxyApp for TestApp {
        type Ctx = ();
        type Tunnel = Cursor<Vec<u8>>;
        fn new_ctx(&self) -> Self::Ctx {}
        async fn upstream(
            &self,
            _ctx: &mut Self::Ctx,
            target: &Target,
        ) -> Result<Option<Self::Tunnel>> {
            assert_eq!(*target, Target::Domain("www.example.com".into(), 80));
            Ok(Some(Cursor::new(vec![])))
        }
        async fn after_forward(&self, _ctx: &mut Self::Ctx, tunnel: Self::Tunnel) -> Result<()> {
            assert_eq!(tunnel.into_inner(), test_data());
            Ok(())
        }
    }

    let server_task = tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:1234").await.unwrap();
        let (stream, addr) = listener.accept().await.unwrap();
        let server = ProxyServer::builder(TestApp)
            .add_proxy(Socks5::default())
            .build()
            .unwrap();
        server.serve(stream, addr).await.unwrap();
    });

    let client_task = tokio::spawn(async move {
        let mut client = Socks5Stream::connect_raw(
            Socks5Command::TCPConnect,
            "127.0.0.1:1234",
            "www.example.com".into(),
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
