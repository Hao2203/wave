use crate::{request::Request, Response};
use anyhow::Result;
use std::future::Future;

pub mod transport;

pub trait FromRequest<'a>: Sized {
    fn from_request(req: Request<'a>) -> impl Future<Output = Result<Self>> + 'a;
}

pub trait IntoResponse<'a> {
    fn into_response(self) -> Response<'a>;
}
