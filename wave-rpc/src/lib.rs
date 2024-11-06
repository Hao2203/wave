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
use futures::{Sink, Stream};
pub use request::Request;
pub use response::Response;
pub use service::Service;
use tokio::io::{AsyncRead, AsyncWrite};

use std::io::Error as IoError;

pub type Result<T, E = IoError> = std::result::Result<T, E>;
pub trait Transport<'a> {
    type Item;

    fn stream(
        &mut self,
        io: impl AsyncRead + Send + Sync + Unpin + 'a,
    ) -> impl Stream<Item = Result<Self::Item>> + Unpin + Send + 'a;

    fn sink(
        &mut self,
        io: impl AsyncWrite + Send + Sync + Unpin + 'a,
    ) -> impl Sink<Result<Self::Item>> + Unpin + Send + 'a;
}
