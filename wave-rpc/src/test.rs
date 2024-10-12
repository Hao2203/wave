#![allow(unused)]

use crate::service::Service;
use anyhow::Result;

struct MyService;

struct AddReq(u32, u32);

// impl Service<AddReq> for MyService {
//     type Response = u32;
//     type Key = ();

//     async fn call(&self, req: AddReq) -> Result<Self::Response> {
//         Ok(req.0 + req.1)
//     }

//     fn key(&self) -> Self::Key {}
// }

// #[tokio::test]
// async fn test() -> Result<()> {
//     let mut server = RpcServer::new();

//     server.register::<MyService, AddReq>(&MyService);

//     Ok(())
// }
