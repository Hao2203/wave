use futures_lite::future;
use std::sync::Arc;

pub trait ContextFactory {
    type Ctx: Send;
    fn create_context(&self) -> Self::Ctx;

    fn cleanup_context(&self, _ctx: Self::Ctx) -> impl std::future::Future<Output = ()> + Send {
        future::ready(())
    }
}

impl ContextFactory for () {
    type Ctx = ();

    fn create_context(&self) -> Self::Ctx {}

    fn cleanup_context(&self, _ctx: Self::Ctx) -> impl std::future::Future<Output = ()> + Send {
        future::ready(())
    }
}
