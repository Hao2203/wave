//! # Examples
//!
//! ```rust
//! use wave_rpc::client::Builder;
//! use wave_rpc::server::{RpcServer, RpcService};
//! use wave_rpc::service::Service;
//! use std::time::Duration;
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
//! use tokio::net::{TcpListener, TcpStream};
//!
//! let rt = tokio::runtime::Builder::new_current_thread()
//!     .enable_all()
//!     .build()
//!     .unwrap();
//! rt.block_on(async move {
//!     let task1 = tokio::spawn(async move {
//!         tokio::time::sleep(Duration::from_secs(1)).await;
//!         let conn = TcpStream::connect("127.0.0.1:8080").await.unwrap();
//!         let builder = Builder::default();
//!         let mut client = builder.build_client(conn).await.unwrap();
//!         let res = client.call::<MyService>(AddReq(1, 2)).await.unwrap();
//!         assert_eq!(res.0, 3);
//!     });

//!     let task2 = tokio::spawn(async move {
//!         let service =
//!             RpcService::with_state(&MyServiceState).register::<MyService>(MyServiceState::add);
//!         let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
//!         let conn = listener.accept().await.unwrap().0;
//!         let server = RpcServer::new(1024);
//!         server.serve(service, conn).await.unwrap();
//!     });
//!     task1.await.unwrap();
//!     task2.await.unwrap();
//! });
//!
//! ```
//!

#![feature(async_closure)]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_traits)]

pub mod body;
mod body_stream;
pub mod client;
pub mod error;
pub mod message;
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
