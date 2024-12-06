#![allow(unused)]
use crate::{
    error::{Error, Result},
    request::Request,
    response::Response,
    transport::Connection,
};
use async_trait::async_trait;
use futures_lite::future::Boxed;
use std::sync::Arc;
use tower::Service;
use tracing::{instrument, trace, Level};

pub mod context;
pub mod fut;
pub mod handler;
// pub mod service;

pub struct RpcServer {
    max_body_size: usize,
}

impl RpcServer {
    pub fn new(max_body_size: usize) -> Self {
        Self { max_body_size }
    }

    // #[instrument(skip_all, level = Level::TRACE, err(level = Level::WARN))]
    pub fn serve<Resp>(
        &self,
        service: impl Service<Request, Response = Response, Error = Error> + Send + Sync,
        mut io: Connection,
    ) -> Boxed<Result<()>> {
        todo!()
    }
}
