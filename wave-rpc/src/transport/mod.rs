use crate::{request::Request, Response};
pub use error::Result;
use std::future::Future;

pub mod error;
#[cfg(feature = "rmp")]
pub mod rmp;

pub trait FromRequest: Sized {
    fn from_request(req: &Request) -> impl Future<Output = Result<Self>> + Send;
}

pub trait IntoRequest {
    fn into_request(self) -> Request;
}
