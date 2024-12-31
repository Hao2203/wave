use crate::Address;
use futures_lite::{future::Boxed as BoxedFuture, FutureExt};
use iroh::{
    endpoint::Connecting,
    protocol::{ProtocolHandler, Router, RouterBuilder},
    Endpoint,
};
use std::{borrow::Cow, fmt::Debug};

pub struct Server {
    endpoint: Endpoint,
    router: Option<RouterBuilder>,
}

impl Server {
    pub fn address(&self) -> Address {
        self.endpoint.node_id().into()
    }

    pub fn add_service<S: Service>(&mut self, service: S) {
        let router = self
            .router
            .take()
            .unwrap_or_else(|| Router::builder(self.endpoint.clone()))
            .accept(service.alpn(), Handler(service));
        self.router = Some(router);
    }
}

#[derive(Debug)]
struct Handler<S>(S);

impl<S> ProtocolHandler for Handler<S>
where
    S: Service + Send + Sync + Debug + 'static,
{
    fn accept(&self, conn: Connecting) -> BoxedFuture<anyhow::Result<()>> {
        let fut = self.0.handle(conn);
        async {
            fut.await;
            Ok(())
        }
        .boxed()
    }
}

pub trait Service: Send + Sync + Debug + 'static {
    fn alpn(&self) -> Cow<'static, [u8]>;

    fn handle(&self, conn: Connecting) -> BoxedFuture<()>;

    fn shutdown(&self) -> BoxedFuture<()> {
        Box::pin(async {})
    }
}
