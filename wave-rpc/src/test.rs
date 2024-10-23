#![allow(unused)]

use std::time::Duration;

use crate::server::RpcService;
use crate::service::Service;

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
    use tokio::task;

    println!("start");
    let task1 = std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let service =
                RpcService::with_state(&MyServiceState).register::<MyService>(MyServiceState::add);
            let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
            println!("listening");
            let conn = listener.accept().await.unwrap().0;
            println!("connected");
            let server = crate::server::RpcServer::new(1024);
            server.serve(service, conn).await.unwrap();
        });
    });
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(1));
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            println!("run task2");
            let conn = TcpStream::connect("127.0.0.1:8080").await.unwrap();
            println!("client connected");
            let mut client = crate::client::RpcBuilder::new(1024)
                .build_client(conn)
                .await
                .unwrap();
            let res = client.call::<MyService>(AddReq(1, 2)).await.unwrap();
            println!("res: {:?}", res.0);
            dbg!(res.0);
            assert_eq!(res.0, 3);
        });
    })
    .join()
    .unwrap();
    task1.join().unwrap()
}

#[test]
fn listening() {
    use tokio::net::{TcpListener, TcpStream};
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .unwrap();
    rt.block_on(async move {
        let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
        println!("listening");
    });
}
