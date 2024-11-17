#![allow(unused)]
use crate::{
    error::Result,
    message::{FromReader, WriteIn},
    request::Request,
    response::Response,
    service::Service,
};
use async_trait::async_trait;
use futures::{AsyncRead, AsyncWrite};
use std::sync::Arc;
use tracing::{instrument, trace, Level};

// pub mod service;

pub struct RpcServer {
    max_body_size: usize,
}

impl RpcServer {
    pub fn new(max_body_size: usize) -> Self {
        Self { max_body_size }
    }

    #[instrument(skip_all, level = Level::TRACE, err(level = Level::WARN))]
    pub async fn serve<Req, Resp>(
        &self,
        service: impl Service<Req, Response = Resp> + Send + Sync,
        mut io: (impl AsyncRead + AsyncWrite + Send + Unpin),
    ) -> Result<()>
    where
        Req: for<'b> FromReader<'b>,
        Resp: WriteIn,
    {
        let req = Req::from_reader(&mut io).await.unwrap();
        let mut resp = service.call(req).await.unwrap();
        resp.write_in(&mut io).await.unwrap();

        Ok(())
    }
}
