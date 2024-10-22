use crate::body::BodyCodec;
use crate::error::Result;
use crate::request::Request;
use crate::request::RequestDecoder;
use crate::response::ResponseEncoder;
use crate::Response;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
pub use service::RpcService;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

// pub mod error;
pub mod code;
pub mod service;

#[async_trait]
pub trait RpcHandler {
    async fn call(&self, req: &mut Request) -> Result<Response>;
}

pub struct RpcServer {
    max_body_size: usize,
}

impl RpcServer {
    pub async fn serve(
        &self,
        service: impl RpcHandler,
        io: (impl AsyncRead + AsyncWrite + Send + Unpin),
    ) -> Result<()> {
        // let (mut io_read, mut io_write) = io;

        let body_codec = BodyCodec::new(self.max_body_size);
        let request_codec = RequestDecoder::new(body_codec);
        let response_codec = ResponseEncoder::new(request_codec);
        let framed = Framed::new(io, response_codec);
        let (mut sink, mut stream) = framed.split();

        while let Some(req) = stream.next().await {
            let mut req = req?;
            let res = service.call(&mut req).await?;
            sink.send(res).await?;
        }

        Ok(())
    }
}
