use crate::{request::Request, Response};
pub use error::Result;
use std::future::Future;

pub mod error;
#[cfg(feature = "rmp")]
pub mod rmp;

pub trait FromRequest: Sized {
    fn from_request(req: Request<'_>) -> impl Future<Output = Result<Self>> + Send;
}

pub trait IntoResponse<'a> {
    fn into_response(self) -> Response<'a>;
}
