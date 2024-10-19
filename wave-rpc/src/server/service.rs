use super::{code::ErrorCode, Result};
use crate::{Body, Request, Response, Service};
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

    pub fn register<State, F, Fut, S>(&mut self, state: &'a State, f: F)
    where
        State: Send + Sync + 'a,
        F: Fn(&State, &mut S::Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<S::Response>> + Send + 'static,
        S: Service + Send + Sync + 'static,
        <S as Service>::Request: FromRequest + Send,
        <S as Service>::Response: ToResponse,
    {
        let id = S::ID;
        self.map.insert(
            id,
            Box::new(FnHandler {
                f,
                state,
                _service: std::marker::PhantomData::<fn() -> S>,
                _future: std::marker::PhantomData,
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

pub struct FnHandler<'a, State, F, Fut, S> {
    f: F,
    state: &'a State,
    _service: std::marker::PhantomData<fn() -> S>,
    _future: std::marker::PhantomData<fn() -> Fut>,
}

#[async_trait]
impl<'a, State, F, Fut, S> Handler for FnHandler<'a, State, F, Fut, S>
where
    State: Send + Sync + 'a,
    Fut: Future<Output = Result<S::Response>> + Send,
    F: Fn(&State, &mut S::Request) -> Fut + Send + Sync,
    S: Service + Send + Sync,
    <S as Service>::Request: FromRequest + Send,
    <S as Service>::Response: ToResponse,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        let mut req = S::Request::from_request(req).await?;
        let resp = (self.f)(self.state, &mut req).await?;
        resp.to_response()
    }
}
