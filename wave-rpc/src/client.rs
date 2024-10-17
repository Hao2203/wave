#![allow(unused)]
use crate::{service::Call, Request, Response, Service};
use tokio::io::{AsyncRead, AsyncWrite};

pub trait MakeConnection {
    fn make_connection(&self) -> (impl AsyncRead + Unpin, impl AsyncWrite + Unpin);
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
    <S as Service>::Request: Send,
    <S as Service>::Response: Send,
    R: AsyncRead + Unpin + Send + Sync,
    W: AsyncWrite + Unpin + Send + Sync,
{
    async fn call(&self, req: <S as Service>::Request) -> anyhow::Result<<S as Service>::Response> {
        todo!()
    }
}
