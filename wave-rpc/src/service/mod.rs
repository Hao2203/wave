use crate::io::{FromAsyncRead, RefReader, RefWriter, WriteResponse};
use anyhow::Result;
use futures::future::BoxFuture;
use std::{collections::HashMap, future::Future, hash::Hash};

pub trait Service<Req> {
    type Response;
    type Key;

    const KEY: Self::Key;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response>> + Send;
}

pub trait Handle<'a> {
    fn handle<'conn>(
        &'a self,
        reader: RefReader<'conn>,
        writer: RefWriter<'conn>,
    ) -> BoxFuture<'conn, Result<()>>
    where
        'a: 'conn;
}

pub struct RpcService<'a, K> {
    map: HashMap<K, Box<dyn Handle<'a> + 'a>>,
}

impl<'a, K> RpcService<'a, K> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register<S, Req>(&mut self, service: &'a S)
    where
        S: Service<Req, Key = K> + Clone + Send + Sync + 'static,
        K: FromAsyncRead + Eq + Hash + Send,
        Req: FromAsyncRead + Send + 'static,
        S::Response: WriteResponse + Send + 'static,
    {
        self.map.insert(S::KEY, ServiceHandler::boxed(service));
    }
}

pub struct ServiceHandler<'a, S, Req> {
    service: &'a S,
    _req: std::marker::PhantomData<Req>,
}

impl<'a, S, Req> ServiceHandler<'a, S, Req> {
    pub fn new(service: &'a S) -> Self {
        Self {
            service,
            _req: std::marker::PhantomData,
        }
    }

    pub fn boxed(service: &'a S) -> Box<dyn Handle<'a> + 'a>
    where
        S: Service<Req> + Clone + Send + Sync + 'static,
        Req: FromAsyncRead + Send + 'static,
        S::Response: WriteResponse + Send + 'static,
    {
        Box::new(ServiceHandler::new(service))
    }
}

impl<'a, S, Req> Handle<'a> for ServiceHandler<'a, S, Req>
where
    S: Service<Req> + Clone + Send + Sync + 'static,
    Req: FromAsyncRead + Send + 'static,
    S::Response: WriteResponse + Send + 'static,
{
    fn handle<'conn>(
        &'a self,
        reader: RefReader<'conn>,
        writer: RefWriter<'conn>,
    ) -> BoxFuture<'conn, Result<()>>
    where
        'a: 'conn,
    {
        let fut = async {
            let req = Req::from_async_read(reader).await?;
            let res = self.service.call(req).await?;
            res.async_write(writer).await?;
            anyhow::Ok(())
        };
        Box::pin(fut)
    }
}
