use super::{Result, RpcHandler};
use crate::{error::Error, service::Version, Body, Request, Response, Service};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, future::Future, sync::Arc};

/// ```rust
/// use wave_rpc::server::RpcService;
/// use wave_rpc::service::Service;
///
/// struct MyService;
///
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct AddReq(u32, u32);
///
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct AddRes(u32);
///
/// impl Service for MyService {
///     type Request = AddReq;
///     type Response = AddRes;
///
///     const ID: u32 = 1;
/// }
///
/// struct MyServiceState;
///
/// impl MyServiceState {
///     async fn add(&self, req: AddReq) -> AddRes {
///         AddRes(req.0 + req.1)
///     }
/// }
///
/// let service = RpcService::with_state(&MyServiceState).register::<MyService>(MyServiceState::add);
/// ```
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

    pub fn register<S>(mut self, f: impl Handle<&'a State, S> + Send + Sync + 'a) -> Self
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
                _service: std::marker::PhantomData::<fn() -> S>,
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
        Err(Error::ServiceNotFound)
    }
}

#[async_trait]
impl<T> RpcHandler for &T
where
    T: RpcHandler + Send + Sync,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        <Self as RpcHandler>::call(self, req).await
    }
}

#[async_trait]
impl<T> RpcHandler for &mut T
where
    T: RpcHandler + Send + Sync,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        <Self as RpcHandler>::call(self, req).await
    }
}

#[async_trait]
impl<T> RpcHandler for Box<T>
where
    T: RpcHandler + Send + Sync,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        <Self as RpcHandler>::call(self, req).await
    }
}

#[async_trait]
impl<T> RpcHandler for Arc<T>
where
    T: RpcHandler + Send + Sync,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        <Self as RpcHandler>::call(self, req).await
    }
}

struct FnHandler<'a, State, F, S> {
    f: F,
    state: &'a State,
    _service: std::marker::PhantomData<fn() -> S>,
}

#[async_trait]
impl<'a, State, F, S> RpcHandler for FnHandler<'a, State, F, S>
where
    State: Send + Sync + 'a,
    F: Handle<&'a State, S> + Send + Sync,
    S: Service + Send + Sync,
    <S as Service>::Request: for<'b> Deserialize<'b> + Send,
    <S as Service>::Response: Serialize + Send,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        let req = req.body().bincode_decode()?;
        let resp = (self.f).call(self.state, req).await;
        let body = Body::bincode_encode(resp)?;
        Ok(Response::success(body))
    }
}

pub trait Handle<State, S: Service> {
    fn call(&self, state: State, req: S::Request) -> impl Future<Output = S::Response> + Send;
}

impl<F, Fut, S, State> Handle<State, S> for F
where
    Fut: Future<Output = S::Response> + Send,
    F: Fn(State, S::Request) -> Fut + Send + Sync,
    S: Service + Send + Sync,
    S::Request: Send,
    S::Response: Send,
    State: Send + Sync,
{
    fn call(&self, state: State, req: S::Request) -> impl Future<Output = S::Response> + Send {
        (self)(state, req)
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
