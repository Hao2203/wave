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
//! let service = RpcService::with_state(&MyServiceState).register::<MyService>(MyServiceState::add);
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
mod test;
// pub mod transport;

pub use body::Body;
pub use request::Request;
pub use response::Response;
pub use service::Service;
