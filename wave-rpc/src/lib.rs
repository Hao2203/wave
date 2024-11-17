#![feature(async_closure)]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_traits)]

// pub mod client;
pub mod body;
pub mod error;
pub mod request;
pub mod response;
pub mod server;
pub mod service;
// #[cfg(test)]
// mod tests;
pub mod code;
pub mod message;

pub use service::ServiceDef;
