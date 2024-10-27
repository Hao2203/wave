use crate::{
    body::BodyCodec,
    error::Result,
    request::{Request, RequestDecoder},
    response::{Response, ResponseEncoder},
};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
pub use service::RpcService;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;
use tracing::{instrument, trace, Level};

pub mod service;

pub struct RpcServer {
    max_body_size: usize,
}

impl RpcServer {
    pub fn new(max_body_size: usize) -> Self {
        Self { max_body_size }
    }

    #[instrument(skip_all, level = Level::TRACE, err(level = Level::WARN))]
    pub async fn serve(
        &self,
        service: impl RpcHandler,
        io: (impl AsyncRead + AsyncWrite + Send + Unpin),
    ) -> Result<()> {
        let body_codec = BodyCodec::new(self.max_body_size);
        let request_codec = RequestDecoder::new(body_codec);
        let response_codec = ResponseEncoder::new(request_codec);
        let framed = Framed::new(io, response_codec);
        let (mut sink, mut stream) = framed.split();

        while let Some(req) = stream.next().await {
            let mut req = req?;

            trace!(
                service_id = req.service_id(),
                service_version = %req.service_version(),
                "start process request"
            );

            let res = service.call(&mut req).await?;
            sink.send(res).await?;

            trace!(
                service_id = req.service_id(),
                service_version = %req.service_version(),
                "finish process request"
            );
        }

        Ok(())
    }
}

#[async_trait]
pub trait RpcHandler {
    async fn call(&self, req: &mut Request) -> Result<Response>;
}

#[async_trait]
impl<T> RpcHandler for &T
where
    T: RpcHandler + Send + Sync,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        <Self as RpcHandler>::call(self, req).await
    }
}

#[async_trait]
impl<T> RpcHandler for &mut T
where
    T: RpcHandler + Send + Sync,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        <Self as RpcHandler>::call(self, req).await
    }
}

#[async_trait]
impl<T> RpcHandler for Box<T>
where
    T: RpcHandler + Send + Sync,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        <Self as RpcHandler>::call(self, req).await
    }
}

#[async_trait]
impl<T> RpcHandler for Arc<T>
where
    T: RpcHandler + Send + Sync,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        <Self as RpcHandler>::call(self, req).await
    }
}
