use super::{ConnHandler, Handle, Result};
use crate::{service::Connection, Service};
use std::future::Future;

pub trait Transport<S> {
    type Handler;

    fn transport<'conn>(
        &self,
        service: S,
    ) -> impl Future<Output = Result<Self::Handler>> + Send + 'conn
    where
        Self: 'conn,
        S: 'conn;
}

pub struct RpcTransport<Codec> {
    codec: Codec,
}

impl<Codec> RpcTransport<Codec> {
    pub fn new(codec: Codec) -> Self {
        Self { codec }
    }
}
