use crate::body::BodyCodec;
use crate::error::Result;
use crate::request::Request;
use crate::request::RequestCodec;
use crate::response::ResponseCodec;
use crate::Response;
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
pub use service::RpcService;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite};

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
        io: (impl AsyncRead + Unpin + Send, impl AsyncWrite + Unpin),
    ) -> Result<()> {
        let (mut io_read, mut io_write) = io;

        let body_codec = BodyCodec::new(self.max_body_size);
        let request_codec = RequestCodec::new(body_codec);
        let response_codec = ResponseCodec::new(body_codec);
        let mut frame_read = FramedRead::new(&mut io_read, request_codec);
        let mut frame_write = FramedWrite::new(&mut io_write, response_codec);

        while let Some(req) = frame_read.next().await {
            let mut req = req?;
            let res = service.call(&mut req).await?;
            frame_write.send(res).await?;
        }

        Ok(())
    }
}
