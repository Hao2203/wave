use crate::{
    request::{BodyType, Header, Request},
    Body,
};
use anyhow::{anyhow, Context, Result};
use async_stream::stream;
use bytes::{Bytes, BytesMut};
use service::RpcService;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use zerocopy::{FromBytes, TryFromBytes};

pub mod service;

pub struct RpcServer {
    max_body_size: usize,
}

impl RpcServer {
    pub async fn serve(
        &self,
        service: impl RpcService,
        mut io: impl AsyncRead + AsyncWrite + Unpin + Send,
    ) -> Result<()> {
        let mut header_buf = [0u8; 40];
        let _ = io.read(&mut header_buf).await?;
        let header: &Header =
            Header::try_ref_from_bytes(&header_buf[..]).map_err(|_| anyhow!(""))?;

        if header.body_size > self.max_body_size as u64 {
            return Err(anyhow!("body size too large"));
        }

        let body: Body = match header.body_type {
            BodyType::Bytes => {
                let mut bytes = BytesMut::with_capacity(header.body_size as usize);
                io.read_buf(&mut bytes).await?;
                Body::Bytes(bytes.into())
            }
            BodyType::Stream => {
                let stream = stream! {
                    let mut bytes = BytesMut::with_capacity(header.body_size as usize);
                    io.read_buf(&mut bytes).await?;
                    yield Ok(Bytes::from(bytes));
                };
                Body::Stream(Box::pin(stream))
            }
        };

        let req = Request { header, body };

        let resp = service.call(req).await?;

        todo!()
    }
}
