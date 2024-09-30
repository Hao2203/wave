use anyhow::Result;
use std::future::Future;

pub trait Service<Req> {
    type Response;
    type Key;

    const KEY: Self::Key;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response>> + Send;
}
