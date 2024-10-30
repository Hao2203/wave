use crate::client::Builder;
use crate::message::Bincode;
use crate::server::{RpcServer, RpcService};
use crate::service::Service;
use std::convert::Infallible;
use std::time::Duration;

#[derive(serde::Serialize, serde::Deserialize)]
struct AddReq(u32, u32);

#[derive(serde::Serialize, serde::Deserialize)]
struct AddRes(u32);

struct AddService;
impl Service for AddService {
    type Request = Bincode<AddReq>;
    type Response = Bincode<AddRes>;

    const ID: u32 = 1;
}

struct AddService2;
impl Service for AddService2 {
    type Request = Bincode<AddReq>;
    type Response = Result<Bincode<AddRes>, Infallible>;

    const ID: u32 = 2;
}

struct MyServiceState;

impl MyServiceState {
    async fn add(&self, req: AddReq) -> AddRes {
        AddRes(req.0 + req.1)
    }

    async fn add2(&self, req: AddReq) -> Result<AddRes, Infallible> {
        Ok(AddRes(req.0 + req.1))
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
            let builder = Builder::default();
            let mut client = builder.build_client(conn).await.unwrap();
            let res = client.call::<AddService>(AddReq(1, 2)).await.unwrap();
            assert_eq!(res.0, 3);

            let res = client
                .call::<AddService2>(AddReq(1, 2))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(res.0, 3);
        });

        let task2 = tokio::spawn(async move {
            let service = RpcService::with_state(&MyServiceState)
                .register::<AddService>(MyServiceState::add)
                .register::<AddService2>(MyServiceState::add2);
            let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
            let conn = listener.accept().await.unwrap().0;
            let server = RpcServer::new(1024);
            server.serve(service, conn).await.unwrap();
        });

        task1.await.unwrap();
        task2.await.unwrap();
    });
}
