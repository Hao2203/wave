#![allow(unused)]
use super::error::Result;
use super::{error::ClientError, Client};
use super::{Builder, Call};
use crate::error::Error;
use crate::{body::BodyCodec, client::to_stream_and_sink, service::Version, Request, Response};
use deadpool::managed::{Manager, Object, PoolError};
use futures::{Sink, Stream};
use std::future::Future;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait MakeConnection {
    type Connection: AsyncRead + AsyncWrite + Send + Sync + Unpin;
    fn make_connection(&self) -> impl Future<Output = Self::Connection> + Send;
}

impl<T> Builder<T>
where
    T: MakeConnection + Sync,
{
    pub fn build(&self) -> Result<Pool<'_, T>> {
        let inner = InnerPool {
            max_body_size: self.max_body_size.unwrap_or(super::DEFAULT_MAX_BODY_SIZE),
            manger: &self.manager,
            service_version: self.version,
        };
        let pool = deadpool::managed::Pool::builder(inner).build()?;

        Ok(Pool { inner: pool })
    }
}

pub struct Pool<'a, T: MakeConnection + Sync> {
    inner: deadpool::managed::Pool<InnerPool<'a, T>>,
}

impl<'a, T: MakeConnection + Sync> Pool<'a, T> {
    pub fn builder(manager: T) -> Builder<T> {
        Builder {
            manager,
            version: Version::default(),
            max_body_size: Default::default(),
        }
    }

    pub async fn client(&self) -> Result<impl Call + Send + Sync + 'a> {
        let pool = self.inner.get().await;
        match pool {
            Ok(mut pool) => Ok(pool),
            Err(e) => {
                if let PoolError::Backend(e) = e {
                    Err(e)
                } else {
                    Err(anyhow::anyhow!("{:?}", e))?
                }
            }
        }
    }
}

impl<T> Call for Object<T>
where
    T: Manager,
    T::Type: Call,
{
    async fn call<S>(&mut self, req: S::Request) -> Result<S::Response>
    where
        S: crate::Service,
        <S as crate::Service>::Request: serde::Serialize + Send,
        <S as crate::Service>::Response: for<'a> serde::Deserialize<'a> + Send,
    {
        self.as_mut().call::<S>(req).await
    }
}

struct InnerPool<'a, T> {
    max_body_size: usize,
    manger: &'a T,
    service_version: Version,
}

impl<'a, T> Manager for InnerPool<'a, T>
where
    T: MakeConnection + Sync,
{
    type Type = PoolClient<'a>;
    type Error = ClientError;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let conn = self.manger.make_connection().await;
        let body_codec = BodyCodec::new(self.max_body_size);
        let io = to_stream_and_sink(conn, body_codec);

        Ok(Client::new(Box::new(io), self.service_version))
    }

    async fn recycle(
        &self,
        obj: &mut Self::Type,
        metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        Ok(())
    }
}
type PoolClient<'a> = Client<Box<dyn super::Transport + Send + Sync + 'a>>;
