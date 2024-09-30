use anyhow::Result;
use std::future::Future;

pub trait Service<Req> {
    type Response;
    type Key;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response>> + Send;

    fn key(&self) -> Self::Key;
}
