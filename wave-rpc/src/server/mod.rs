use crate::{request::Request, Response};
use anyhow::Result;
use std::future::Future;

pub mod service;
pub mod transport;

pub trait FromRequest<'a>: Sized {
    fn from_request(req: &'a mut Request<'a>) -> impl Future<Output = Result<Self>> + Send;
}

pub trait IntoResponse<'a> {
    fn into_response(self) -> Response<'a>;
}

pub struct RpcServer {}
