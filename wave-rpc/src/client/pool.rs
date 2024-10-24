#![allow(unused)]
use crate::{body::BodyCodec, client::to_stream_and_sink, service::Version};

use super::{error::ClientError, Client};
use deadpool::managed::Manager;
use std::future::Future;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait MakeConnection {
    type Connection: Connection;
    fn make_connection(&self) -> impl Future<Output = Self::Connection> + Send;
}

pub trait Connection: AsyncRead + AsyncWrite + Send + Sync + Unpin {}

impl<T> Connection for T where T: AsyncRead + AsyncWrite + Send + Sync + Unpin {}

struct Conn<T> {
    io: T,
}

pub struct Pool<T> {
    max_body_size: usize,
    manger: T,
    service_version: Version,
}

impl<T> Manager for Pool<T>
where
    T: MakeConnection + Send + Sync,
{
    type Type = Client<impl Stream<Item = Result<Response>> + Sink<Request, Error = ClientError>>;
    type Error = ClientError;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let conn = self.manger.make_connection().await;
        let body_codec = BodyCodec::new(self.max_body_size);
        let io = to_stream_and_sink(conn, body_codec);

        Ok(Client::new(io, self.service_version))
    }

    async fn recycle(
        &self,
        obj: &mut Self::Type,
        metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        Ok(())
    }
}
