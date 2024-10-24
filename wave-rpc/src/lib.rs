//! # Examples
//!
//! ```rust
//! use wave_rpc::server::RpcService;
//! use wave_rpc::service::Service;
//!
//! struct MyService;
//!
//! #[derive(serde::Serialize, serde::Deserialize)]
//! struct AddReq(u32, u32);
//!
//! #[derive(serde::Serialize, serde::Deserialize)]
//! struct AddRes(u32);
//!
//! impl Service for MyService {
//!     type Request = AddReq;
//!     type Response = AddRes;
//!
//!     const ID: u32 = 1;
//! }
//!
//! struct MyServiceState;
//!
//! impl MyServiceState {
//!     async fn add(&self, req: AddReq) -> AddRes {
//!         AddRes(req.0 + req.1)
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     use tokio::net::{TcpListener, TcpStream};
//!     use tokio::task;
//!
//!     dbg!("starting");
//!     let task1 = task::spawn(async move {
//!         let service = RpcService::with_state(&MyServiceState).register::<MyService>(MyServiceState::add);
//!         let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
//!         dbg!("listening");
//!         let conn = listener.accept().await.unwrap().0;
//!         dbg!("connected");
//!         let server = wave_rpc::server::RpcServer::new(1024);
//!         server.serve(service, conn).await.unwrap();
//!     });
//!     let task2 = task::spawn(async move {
//!         let conn = TcpStream::connect("127.0.0.1:8080").await.unwrap();
//!         dbg!("client connected");
//!         let mut client = wave_rpc::client::RpcBuilder::new(1024).build_client(conn).await.unwrap();
//!         let res = client.call::<MyService>(AddReq(1, 2)).await.unwrap();
//!         assert_eq!(res.0, 3);
//!     });
//!     task1.await.unwrap();
//!     task2.await.unwrap();
//! }
//!
//! ```
//!

pub mod body;
pub mod client;
pub mod error;
pub mod request;
pub mod response;
pub mod server;
pub mod service;
#[cfg(test)]
mod tests;
// pub mod transport;

pub use body::Body;
pub use request::Request;
pub use response::Response;
pub use service::Service;
