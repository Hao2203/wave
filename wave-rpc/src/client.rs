#![allow(unused)]
use crate::{Request, Response, Service};
use tokio::io::{AsyncRead, AsyncWrite};

pub trait MakeConnection {
    fn make_connection(&self) -> (impl AsyncRead + Unpin, impl AsyncWrite + Unpin);
}

pub struct RpcClient<T> {
    pub max_body_size: usize,
    pub codec: T,
}
