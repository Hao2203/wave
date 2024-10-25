#![allow(unused)]

use crate::client::{Builder, Call, Client};
use crate::server::{RpcServer, RpcService};
use crate::service::Service;
use std::time::Duration;

struct MyService;

#[derive(serde::Serialize, serde::Deserialize)]
struct AddReq(u32, u32);

#[derive(serde::Serialize, serde::Deserialize)]
struct AddRes(u32);

impl Service for MyService {
    type Request = AddReq;
    type Response = AddRes;

    const ID: u32 = 1;
}

struct MyServiceState;

impl MyServiceState {
    async fn add(&self, req: AddReq) -> AddRes {
        AddRes(req.0 + req.1)
    }
}

#[test]
fn test() {
    use tokio::net::{TcpListener, TcpStream};

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        let task1 = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let conn = TcpStream::connect("127.0.0.1:8080").await.unwrap();
            let mut client = Builder::default().build_client(conn).await.unwrap();
            let res = client.call::<MyService>(AddReq(1, 2)).await.unwrap();
            assert_eq!(res.0, 3);
        });

        let task2 = tokio::spawn(async move {
            let service =
                RpcService::with_state(&MyServiceState).register::<MyService>(MyServiceState::add);
            let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
            let conn = listener.accept().await.unwrap().0;
            let server = RpcServer::new(1024);
            server.serve(service, conn).await.unwrap();
        });

        task1.await.unwrap();
        task2.await.unwrap();
    });
}
