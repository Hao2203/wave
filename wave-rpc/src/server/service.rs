use super::Result;
use super::{FromRequest, IntoResponse};
use crate::{Request, Response, Service};
use futures::future::BoxFuture;

pub trait RpcService<'a> {
    fn call(&'a self, req: Request<'a>) -> BoxFuture<'a, Result<Response<'a>>>;
}

impl<'a, T> RpcService<'a> for T
where
    T: Service + Sync + 'a,
    <T as Service>::Request: FromRequest + Send,
    <T as Service>::Response: IntoResponse<'a> + Send,
{
    fn call(&'a self, req: Request<'a>) -> BoxFuture<'a, Result<Response<'a>>> {
        let fut = async move {
            let req = T::Request::from_request(req).await?;

            let resp = T::call(self, req).await?;

            Ok(resp.into_response())
        };
        Box::pin(fut)
    }
}
