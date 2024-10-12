use super::Result;
use super::{FromRequest, IntoResponse};
use crate::{Request, Response, Service};
use futures::future::BoxFuture;

pub trait RpcService<'a> {
    fn call(&'a self, req: &'a mut Request<'a>) -> BoxFuture<'a, Result<Response<'a>>>;
}

impl<'a, T> RpcService<'a> for T
where
    T: Service + Sync + 'a,
    <T as Service>::Request<'a>: FromRequest<'a> + Send,
    <T as Service>::Response<'a>: IntoResponse<'a> + Send,
{
    fn call(&'a self, req: &'a mut Request<'a>) -> BoxFuture<'a, Result<Response<'a>>> {
        let fut = async move {
            let req = T::Request::from_request(req).await?;

            let resp = T::call(self, req).await?;

            // Ok(resp.into_response())
            todo!()
        };
        Box::pin(fut)
    }
}
