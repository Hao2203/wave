#![allow(unused)]
use crate::{error::Result, Request, Response, Service};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};

pub mod pool;

pub trait Call<S: Service> {
    fn call(
        &self,
        req: S::Request,
    ) -> impl std::future::Future<Output = Result<S::Response>> + Send;
}

pub struct RpcClient<T> {
    pub max_body_size: usize,
    pub codec: T,
}

pub struct Caller<R, W> {
    io_read: R,
    io_write: W,
}

impl<R, W, S> Call<S> for Caller<R, W>
where
    S: Service,
    <S as Service>::Request: Serialize + Send,
    <S as Service>::Response: for<'a> Deserialize<'a> + Send,
    R: AsyncRead + Unpin + Send + Sync,
    W: AsyncWrite + Unpin + Send + Sync,
{
    async fn call(
        &self,
        req: <S as Service>::Request,
    ) -> crate::error::Result<<S as Service>::Response> {
        todo!()
    }
}
