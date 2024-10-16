#![allow(unused)]
use crate::{
    transport::{RequestCodec, ResponseCodec},
    Request, Response, Service,
};
use tokio::io::{AsyncRead, AsyncWrite};

pub trait MakeConnection {
    fn make_connection(&self) -> (impl AsyncRead + Unpin, impl AsyncWrite + Unpin);
}

pub struct RpcClient<T> {
    pub max_body_size: usize,
    pub codec: T,
}

impl<T> RpcClient<T> {
    pub async fn call<S: Service>(&self, req: S::Request) -> anyhow::Result<S::Response>
    where
        T: RequestCodec<S::Request> + ResponseCodec<S::Response>,
    {
        todo!()
    }
}
