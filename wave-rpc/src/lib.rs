//! # Examples
//!
//! ```rust
//! use wave_rpc::service::Service;
//! use std::future::Future;
//! use anyhow::Result;
//! use server::RpcServer;
//!
//! struct MyService;
//!
//! struct AddReq(u32, u32);
//!
//! impl Service<AddReq> for MyService {
//!     type Response = u32;
//!     type Key = ();
//!
//!     const KEY: Self::Key = ();
//!
//!     async fn call(&self, req: AddReq) -> Result<Self::Response> {
//!         Ok(req.0 + req.1)
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let mut server = RpcServer::new();
//!
//!     server.register::<MyService, AddReq>(&MyService);
//!
//!     Ok(())
//! }
//!
//! ```
//!
//!
//!

pub mod body;
pub mod client;
pub mod codec;
pub mod error;
pub mod request;
pub mod response;
pub mod server;
pub mod service;
#[cfg(test)]
mod test;
// pub mod transport;

pub use body::Body;
pub use request::Request;
pub use response::Response;
pub use service::Service;
