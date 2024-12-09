use std::sync::Arc;

pub trait ContextFactory<Ctx> {
    fn create_context(&self) -> Ctx;
}

impl<T, Ctx> ContextFactory<Ctx> for Arc<T>
where
    T: ContextFactory<Ctx>,
{
    fn create_context(&self) -> Ctx {
        T::create_context(self)
    }
}
