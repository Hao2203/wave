use super::{ConnHandler, Handle, Result};
use crate::{service::Connection, Service};
use std::future::Future;

pub trait Transport<S, Req>
where
    S: Service<Req>,
{
    type Handler<'conn>
    where
        S: 'conn,
        Self: 'conn;

    fn transport<'a: 'conn, 'conn>(
        &'a self,
        service: &'a S,
    ) -> impl Future<Output = Result<Self::Handler<'conn>>> + Send;
}

pub struct RpcTransport<Codec> {
    codec: Codec,
}

impl<Codec> RpcTransport<Codec> {
    pub fn new(codec: Codec) -> Self {
        Self { codec }
    }
}
impl<S, Req, Codec> Transport<S, Req> for RpcTransport<Codec>
where
    S: Service<Req> + Send + Sync + 'static,
    Req: Send + 'static,
    S::Response: Send + 'static,
    Codec: crate::codec::CodecRead<Req> + crate::codec::CodecWrite<S::Response> + Send + Sync,
{
    type Handler<'conn> = Box<dyn Handle<'conn, dyn Connection + Unpin + Send> + 'conn> where Codec: 'conn;

    async fn transport<'a: 'conn, 'conn>(&'a self, service: &'a S) -> Result<Self::Handler<'conn>> {
        let handler = ConnHandler::boxed(service, &self.codec);
        Ok(handler)
    }
}
