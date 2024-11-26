#![allow(unused)]
use crate::{
    error::{Error, Result},
    message::{FromReader, SendTo},
    request::Request,
    response::Response,
    service::Service,
};
use async_trait::async_trait;
use futures::{AsyncRead, AsyncWrite};
use std::sync::Arc;
use tracing::{instrument, trace, Level};

pub mod context;
pub mod service;

pub struct RpcServer {
    max_body_size: usize,
}

impl RpcServer {
    pub fn new(max_body_size: usize) -> Self {
        Self { max_body_size }
    }

    #[instrument(skip_all, level = Level::TRACE, err(level = Level::WARN))]
    pub async fn serve<'a, Req, Resp>(
        &self,
        service: impl Service<Req, Response = Resp, Error = Error> + Send + Sync + 'a,
        mut io: (impl AsyncRead + AsyncWrite + Send + Unpin),
    ) -> Result<()>
    where
        Req: for<'b> FromReader<'b, Error: Into<Error>>,
        Resp: SendTo<Error: Into<Error>>,
    {
        let req = Req::from_reader(&mut io).await.map_err(Into::into)?;
        let mut res = service.call(req).await?;
        res.send_to(&mut io).await.map_err(Into::into)?;

        Ok(())
    }
}

pub trait Io: AsyncRead + AsyncWrite + Send + Unpin {}

impl<T> Io for T where T: AsyncRead + AsyncWrite + Send + Unpin {}
