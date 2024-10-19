use super::{code::ErrorCode, Result};
use crate::{service::Call, Body, Request, Response, Service};
use async_trait::async_trait;
use std::{collections::BTreeMap, future::Future};

#[cfg(feature = "bincode")]
pub mod bincode;

#[async_trait]
pub trait Handler {
    async fn call(&self, req: &mut Request) -> Result<Response>;
}

pub trait FromRequest: Sized {
    fn from_request(req: &mut Request) -> impl Future<Output = Result<Self>> + Send;
}

pub trait ToResponse {
    fn to_response(&self) -> Result<Response>;
}

pub struct RpcHandler<'a, T, S> {
    caller: &'a T,
    _service: std::marker::PhantomData<fn() -> S>,
}

#[async_trait]
impl<'a, T, S> Handler for RpcHandler<'a, T, S>
where
    S: Service,
    <S as Service>::Request: FromRequest + Send,
    <S as Service>::Response: ToResponse,
    T: Call<S> + Send + Sync,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        let req = S::Request::from_request(req).await?;
        let resp = T::call(self.caller, req).await?;
        resp.to_response()
    }
}

pub struct RpcService<'a> {
    map: BTreeMap<u64, Box<dyn Handler + Sync + 'a>>,
}

impl<'a> RpcService<'a> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn register<S: Service + 'static>(&mut self, caller: &'a (impl Call<S> + Send + Sync))
    where
        <S as Service>::Request: FromRequest + Send,
        <S as Service>::Response: ToResponse,
    {
        let id = S::ID;
        self.map.insert(
            id,
            Box::new(RpcHandler {
                caller,
                _service: std::marker::PhantomData,
            }),
        );
    }
}

#[async_trait]
impl<'a> Handler for RpcService<'a> {
    async fn call(&self, req: &mut Request) -> Result<Response> {
        let id = req.header.service_id;
        if let Some(handler) = self.map.get(&id) {
            return handler.call(req).await;
        }
        Ok(Response::new(
            ErrorCode::ServiceNotFound as u16,
            Body::new_empty(),
        ))
    }
}
