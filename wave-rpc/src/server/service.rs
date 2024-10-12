use super::Result;
use super::{FromRequest, IntoResponse};
use crate::{Request, Response, Service};
use futures::future::BoxFuture;

pub trait RpcService {
    fn call<'a>(&'a self, req: Request<'a>) -> BoxFuture<'_, Result<Response<'a>>>;
}

impl<T> RpcService for T
where
    for<'a> T: Service<'a, Request: FromRequest<'a>, Response: IntoResponse<'a>> + Sync + 'a,
    for<'a> <T as Service<'a>>::Request: FromRequest<'a> + Send,
    for<'a> <T as Service<'a>>::Response: IntoResponse<'a> + Send,
{
    fn call<'a>(&'a self, req: Request<'a>) -> BoxFuture<'_, Result<Response<'a>>> {
        let fut = async {
            let req = T::Request::from_request(req).await?;

            let resp = T::call(self, req).await?;

            Ok(resp.into_response())
        };
        Box::pin(fut)
    }
}
