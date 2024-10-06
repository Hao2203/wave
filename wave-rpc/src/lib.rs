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

pub mod codec;
pub mod proto;
pub mod server;
pub mod service;
#[cfg(test)]
mod test;

pub use service::Service;
