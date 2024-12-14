#![allow(unused)]
use crate::{
    body::{Body, Frame},
    error::{BoxError, Error, Result},
    request::{Header, Request},
    response::Response,
    transport::{Connection, Transport},
};
use async_compat::CompatExt;
use async_trait::async_trait;
use futures_lite::{future::Boxed, io, AsyncRead, AsyncWrite, StreamExt};
use std::{future::Future, pin::pin, sync::Arc};
use tokio_util::codec::FramedRead;
use tower::Service;
use tracing::{instrument, trace, Level};

pub mod fut;
pub mod service;

pub trait ServerApp {
    fn process_connection(
        self: &Arc<Self>,
        conn: Box<dyn Transport>,
    ) -> impl Future<Output = Option<Box<dyn Transport>>> + Send;
}

#[derive(Debug, Clone)]
pub struct RpcServer {
    max_body_size: usize,
}

impl RpcServer {
    pub fn new(max_body_size: usize) -> Self {
        Self { max_body_size }
    }

    // #[instrument(skip_all, level = Level::TRACE, err(level = Level::WARN))]
    pub fn serve<Resp, S>(
        &self,
        mut service: S,
        mut io: impl AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
    ) -> impl Future<Output = Result<(), BoxError>> + Send + 'static
    where
        <S as Service<Request>>::Future: std::marker::Send,
        S: Service<Request, Response = Response, Error = Error> + Send + Sync + 'static,
    {
        let (mut reader, mut writer) = io::split(io);

        async move {
            let header = Header::from_reader(&mut reader).await?;
            let body = Body::from_reader(reader);
            let req = todo!();
            let res = service.call(req).await?;
            res.write_into(&mut writer).await?;

            Ok(())
        }
    }
}
