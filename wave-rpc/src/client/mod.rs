#![allow(unused)]
use crate::{Request, Response, Service};
use error::{ClientError, Result};
use futures::{Sink, SinkExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};

pub mod error;
pub mod pool;

pub trait Call<S: Service> {
    fn call(
        &mut self,
        req: S::Request,
    ) -> impl std::future::Future<Output = Result<S::Response>> + Send;
}

pub struct RpcClient<T> {
    pub max_body_size: usize,
    pub codec: T,
}

pub struct Caller<T> {
    io: T,
    service_version: u32,
}

impl<T, S> Call<S> for Caller<T>
where
    S: Service,
    <S as Service>::Request: Serialize + Send,
    <S as Service>::Response: for<'a> Deserialize<'a> + Send,
    T: Stream<Item = Response> + Sink<Request, Error = std::io::Error> + Send + Sync + Unpin,
{
    async fn call(&mut self, req: <S as Service>::Request) -> Result<<S as Service>::Response> {
        let req = Request::new::<S>(req, self.service_version)?;
        self.io.send(req).await?;
        let res = self
            .io
            .next()
            .await
            .ok_or_else(|| ClientError::ReceiveResponseFailed)?;
        if !res.is_success() {
            Err(ClientError::ErrorWithCode(res.code()))?;
        }
        let res = res.into_body().bincode_decode()?;
        Ok(res)
    }
}
