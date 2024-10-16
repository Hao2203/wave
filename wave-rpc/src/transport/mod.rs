use crate::{request::Request, server::service::Handler, service::Call, Response, Service};
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

pub trait RequestCodec<T> {
    fn decode(&self, req: &mut Request<'_>) -> impl Future<Output = Result<T>> + Send;

    fn code(&self, req: T) -> Request<'_>;
}

pub trait ResponseCodec<T> {
    fn decode(&self, resp: &mut Response<'_>) -> impl Future<Output = Result<T>> + Send;

    fn code(&self, resp: T) -> Response<'_>;
}

pub trait RpcCore<S: Service> {
    fn service_id(&self, service: &S) -> u64;

    fn register(
        &mut self,
        service: &S,
        call: &impl Call<S>,
    ) -> impl Future<Output = Result<()>> + Send;

    fn get_handler(&self, service_id: u64)
        -> impl Future<Output = Result<Box<dyn Handler>>> + Send;
}
