use crate::service::Call;
use crate::transport::{FromRequest, IntoResponse, Result};
use crate::{Request, Response, Service};
use async_trait::async_trait;

#[async_trait]
pub trait RpcService {
    async fn call(&self, req: Request<'_>) -> Result<Response<'_>>;
}

#[async_trait]
impl<T> RpcService for T
where
    T: Service + Call<T> + Sync,
    <T as Service>::Request: FromRequest + Send,
    for<'a> <T as Service>::Response: IntoResponse<'a> + Send,
{
    async fn call(&self, req: Request<'_>) -> Result<Response<'_>> {
        let req = T::Request::from_request(req).await?;

        let resp = T::call(self, req).await?;

        Ok(resp.into_response())
    }
}
