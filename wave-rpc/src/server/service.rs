use super::{Result, RpcHandler};
use crate::{
    error::{Code, Error},
    service::Version,
    Body, Request, Response, Service,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, future::Future};

pub struct RpcService<'a, S> {
    map: BTreeMap<ServiceKey, Box<dyn RpcHandler + Send + Sync + 'a>>,
    state: &'a S,
    version: Version,
}

impl RpcService<'_, ()> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            state: &(),
            version: Default::default(),
        }
    }
}
impl<'a, State> RpcService<'a, State> {
    pub fn with_state(state: &'a State) -> Self {
        Self {
            map: BTreeMap::new(),
            state,
            version: Default::default(),
        }
    }

    pub fn set_state<State2>(self, state: &'a State2) -> RpcService<'a, State2> {
        RpcService {
            map: self.map,
            state,
            version: self.version,
        }
    }

    pub fn clear_state(self) -> RpcService<'a, ()> {
        self.set_state(&())
    }

    pub fn version(mut self, version: impl Into<Version>) -> Self {
        self.version = version.into();
        self
    }

    pub fn register<S>(
        mut self,
        f: impl Handle<&'a State, S::Request, Response = S::Response> + Send + Sync + 'a,
    ) -> Self
    where
        State: Send + Sync + 'a,
        S: Service + Send + Sync + 'static,
        <S as Service>::Request: for<'b> Deserialize<'b> + Send,
        <S as Service>::Response: Serialize + Send,
    {
        let id = S::ID;
        let key = ServiceKey::new(id, self.version);
        self.map.insert(
            key,
            Box::new(FnHandler {
                f,
                state: self.state,
                _service: std::marker::PhantomData,
            }),
        );
        self
    }

    pub fn merge<State2>(mut self, other: RpcService<'a, State2>) -> Self {
        self.map.extend(other.map);
        self
    }

    pub fn merge_with_state<State2>(
        mut self,
        other: RpcService<'a, State2>,
    ) -> RpcService<'a, State2>
    where
        State2: Send + Sync + 'a,
    {
        self.map.extend(other.map);
        self.set_state(other.state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServiceKey {
    pub id: u32,
    pub version: Version,
}

impl ServiceKey {
    pub fn new(id: u32, version: impl Into<Version>) -> Self {
        Self {
            id,
            version: version.into(),
        }
    }
}

#[async_trait]
impl<'a, State> RpcHandler for RpcService<'a, State>
where
    State: Send + Sync + 'a,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        let id = req.header.service_id;
        let version = req.header.service_version;
        let key = ServiceKey::new(id, version);
        if let Some(handler) = self.map.get(&key) {
            return handler.call(req).await;
        }
        Ok(Response::new(
            Code::ServiceNotFound as u16,
            Body::new_empty(),
        ))
    }
}

struct FnHandler<'a, State, F, Req, Resp> {
    f: F,
    state: &'a State,
    _service: std::marker::PhantomData<fn() -> (Req, Resp)>,
}

#[async_trait]
impl<'a, State, F, Req, Resp> RpcHandler for FnHandler<'a, State, F, Req, Resp>
where
    State: Send + Sync + 'a,
    F: Handle<&'a State, Req, Response = Resp> + Send + Sync,
    Req: for<'b> Deserialize<'b> + Send,
    Resp: Serialize + Send,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        let req = req.body().bincode_decode()?;
        let resp = (self.f)
            .call(self.state, req)
            .await
            .map_err(|e| Error::HandleError(e.into()))?;
        let body = Body::bincode_encode(resp)?;
        Ok(Response::success(body))
    }
}

pub trait Handle<State, Req> {
    type Error: core::error::Error + Send + Sync + 'static;
    type Response;
    fn call(
        &self,
        state: State,
        req: Req,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send;
}

impl<F, Fut, E, Req, Resp, State> Handle<State, Req> for F
where
    Fut: Future<Output = Result<Resp, E>> + Send,
    E: core::error::Error + Send + Sync + 'static,
    F: Fn(State, Req) -> Fut + Send + Sync,
    State: Send + Sync,
    Req: Send + Sync,
    Resp: Send + Sync,
{
    type Error = E;
    type Response = Resp;
    async fn call(&self, state: State, req: Req) -> Result<Resp, Self::Error> {
        (self)(state, req).await
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        is_send::<RpcService<'_, ()>>();
    }

    fn is_send<T: Send>() {}
}
