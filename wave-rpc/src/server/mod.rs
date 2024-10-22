#![allow(unused)]
use std::future::Future;

use crate::error::{Error, Result};
use crate::Response;
use crate::{
    request::{Header, Request},
    Body,
};
use async_stream::stream;
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use futures::StreamExt;
pub use service::RpcService;

// pub mod error;
pub mod code;
pub mod request;
pub mod response;
pub mod service;

#[async_trait]
pub trait RpcHandler {
    async fn call(&self, req: Request) -> Result<Response>;
}

pub struct RpcServer {
    max_body_size: usize,
}

// impl RpcServer {
//     pub async fn serve(
//         &self,
//         service: impl Handler,
//         io: (impl AsyncRead + Unpin + Send, impl AsyncWrite + Unpin),
//     ) -> Result<()> {
//         let (mut io_read, mut io_write) = io;

//         let header = &Header::from_reader(&mut io_read).await?;

//         if header.body_size > self.max_body_size as u64 {
//             return Err(anyhow!("body size too large"));
//         }

//         let body: Body = match header.body_type {
//             BodyType::Bytes => {
//                 let mut bytes = BytesMut::with_capacity(header.body_size as usize);
//                 io_read.read_buf(&mut bytes).await?;
//                 Body::Bytes(bytes.into())
//             }
//             BodyType::Stream => {
//                 let stream = stream! {
//                     for _ in 0..u32::MAX {
//                         let mut bytes = BytesMut::with_capacity(header.body_size());
//                         io_read.read_buf(&mut bytes).await?;
//                         yield Ok(Bytes::from(bytes));
//                     }
//                 };
//                 Body::Stream(Box::pin(stream))
//             }
//         };

//         let req = Request { header, body };

//         let mut resp = service.call(req).await?;

//         io_write.write_all(resp.header().as_bytes()).await?;

//         while let Some(bytes) = resp.body_mut().next().await {
//             io_write.write_all(&bytes?).await?;
//         }
//         io_write.shutdown().await?;

//         Ok(())
//     }
// }
