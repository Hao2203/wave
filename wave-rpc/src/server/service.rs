use std::{collections::BTreeMap, sync::Arc};

use crate::{
    service::Call,
    transport::{FromRequest, IntoResponse},
    Request, Response, Service,
};
use async_trait::async_trait;

#[async_trait]
pub trait Handler {
    async fn call(&self, req: Request<'_>) -> anyhow::Result<Response<'_>>;
}

pub struct RpcHandler<T, S> {
    caller: Arc<T>,
    _service: std::marker::PhantomData<fn() -> S>,
}

#[async_trait]
impl<T, S> Handler for RpcHandler<T, S>
where
    S: Service,
    <S as Service>::Request: FromRequest + Send,
    <S as Service>::Response: IntoResponse + Send,
    T: Call<S> + Send + Sync,
{
    async fn call(&self, req: Request<'_>) -> anyhow::Result<Response<'_>> {
        let req = S::Request::from_request(&req).await?;
        let resp = T::call(&self.caller, req).await?;
        Ok(S::Response::into_response(resp))
    }
}

pub struct RpcService<T> {
    map: BTreeMap<u64, Box<dyn Handler + Sync>>,
    caller: Arc<T>,
}

impl<T> RpcService<T> {
    pub fn new(caller: Arc<T>) -> Self {
        Self {
            map: BTreeMap::new(),
            caller,
        }
    }

    pub fn register<S: Service + 'static>(&mut self)
    where
        T: Call<S> + Send + Sync + 'static,
        <S as Service>::Request: FromRequest + Send,
        <S as Service>::Response: IntoResponse + Send,
    {
        let id = S::ID;
        self.map.insert(
            id,
            Box::new(RpcHandler::<T, S> {
                caller: self.caller.clone(),
                _service: std::marker::PhantomData,
            }),
        );
    }
}

#[async_trait]
impl<T> Handler for RpcService<T>
where
    T: Send + Sync,
{
    async fn call(&self, req: Request<'_>) -> anyhow::Result<Response<'_>> {
        let id = req.header.service_id;
        if let Some(handler) = self.map.get(&id) {
            return handler.call(req).await;
        }
        Err(anyhow::anyhow!("unknown handler"))
    }
}
