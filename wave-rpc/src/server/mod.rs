use crate::{
    request::{Header, Request},
    Response,
};
use anyhow::{anyhow, Context, Result};
use bytes::{Bytes, BytesMut};
use service::RpcService;
use std::future::Future;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use zerocopy::FromBytes;

pub mod service;

pub struct RpcServer {
    max_body_size: usize,
}

impl RpcServer {
    pub async fn serve(
        &self,
        service: impl RpcService,
        mut io: impl AsyncRead + AsyncWrite + Unpin,
    ) -> Result<()> {
        let mut buf = [0u8; 40];
        // let _ = io.read(&mut buf).await?;
        let header: &Header = Header::ref_from_bytes(&buf[..]).map_err(|_| anyhow!(""))?;

        Ok(())
    }
}
