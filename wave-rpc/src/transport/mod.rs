use crate::{request::Request, Response};
pub use error::Result;
use std::future::Future;

pub mod error;
#[cfg(feature = "rmp")]
pub mod rmp;

pub trait FromRequest: Sized {
    fn from_request(req: &Request<'_>) -> impl Future<Output = Result<Self>> + Send;
}

pub trait IntoRequest {
    fn into_request<'a>(self) -> Request<'a>
    where
        Self: 'a;
}

pub trait FromResponse {
    fn from_response(resp: Response<'_>) -> Result<Self>
    where
        Self: Sized;
}

pub trait IntoResponse {
    fn into_response<'a>(self) -> Response<'a>
    where
        Self: 'a;
}
