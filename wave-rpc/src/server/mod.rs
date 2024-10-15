use crate::{
    body::BodyType,
    request::{Header, Request},
    Body,
};
use anyhow::{anyhow, Result};
use async_stream::stream;
use bytes::{Bytes, BytesMut};
use futures::StreamExt;
use service::RpcService;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use zerocopy::{IntoBytes, TryFromBytes};

pub mod service;

pub struct RpcServer {
    max_body_size: usize,
}

impl RpcServer {
    pub async fn serve(
        &self,
        service: impl RpcService,
        io: (impl AsyncRead + Unpin + Send, impl AsyncWrite + Unpin),
    ) -> Result<()> {
        let (mut io_read, mut io_write) = io;

        let mut header_buf = [0u8; 40];
        let _ = io_read.read(&mut header_buf).await?;
        let header: &Header =
            Header::try_ref_from_bytes(&header_buf[..]).map_err(|_| anyhow!(""))?;

        if header.body_size > self.max_body_size as u64 {
            return Err(anyhow!("body size too large"));
        }

        let body: Body = match header.body_type {
            BodyType::Bytes => {
                let mut bytes = BytesMut::with_capacity(header.body_size as usize);
                io_read.read_buf(&mut bytes).await?;
                Body::Bytes(bytes.into())
            }
            BodyType::Stream => {
                let stream = stream! {
                    for _ in 0..u32::MAX {
                        let mut bytes = BytesMut::with_capacity(header.body_size as usize);
                        io_read.read_buf(&mut bytes).await?;
                        yield Ok(Bytes::from(bytes));
                    }

                };
                Body::Stream(Box::pin(stream))
            }
        };

        let req = Request { header, body };

        let mut resp = service.call(req).await?;

        io_write.write_all(resp.header().as_bytes()).await?;

        while let Some(bytes) = resp.body_mut().next().await {
            io_write.write_all(&bytes?).await?;
        }
        io_write.shutdown().await?;

        Ok(())
    }
}
