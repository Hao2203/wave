use super::{code::ErrorCode, Result};
use crate::{Body, Request, Response, Service};
use async_trait::async_trait;
use futures::future::BoxFuture;
use std::{collections::BTreeMap, future::Future};

#[cfg(feature = "bincode")]
pub mod bincode;

pub trait RpcHandler {
    fn call(&self, req: Request) -> BoxFuture<Result<Response>>;
}

pub trait FromRequest: Sized {
    fn from_request(req: &mut Request) -> impl Future<Output = Result<Self>> + Send;
}

pub trait ToResponse {
    fn to_response(&self) -> Result<Response>;
}

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
/// let service = RpcService::new().register::<MyService, _, _, _>(&MyServiceState, MyServiceState::add);
/// ```
pub struct RpcService<'a> {
    map: BTreeMap<ServiceKey, Box<dyn RpcHandler + Sync + 'a>>,
    version: u32,
}

impl<'a> RpcService<'a> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            version: 0,
        }
    }

    pub fn version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    pub fn register<S, State, F, Fut>(mut self, state: &'a State, f: F) -> Self
    where
        State: Send + Sync + 'a,
        F: Fn(&'a State, S::Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = S::Response> + Send + 'static,
        S: Service + Send + Sync + 'static,
        <S as Service>::Request: FromRequest + Send,
        <S as Service>::Response: ToResponse,
    {
        let id = S::ID;
        let key = ServiceKey::new(id, self.version);
        self.map.insert(
            key,
            Box::new(FnHandler {
                f,
                state,
                _service: std::marker::PhantomData::<fn() -> S>,
                _future: std::marker::PhantomData,
            }),
        );
        self
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.map.extend(other.map);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServiceKey {
    pub id: u32,
    pub version: u32,
}

impl ServiceKey {
    pub fn new(id: u32, version: u32) -> Self {
        Self { id, version }
    }
}

impl<'a> RpcHandler for RpcService<'a> {
    fn call(&self, req: &mut Request) -> BoxFuture<'_, Result<Response>> {
        let fut = async {
            let id = req.header.service_id;
            let version = req.header.service_version;
            let key = ServiceKey::new(id, version);
            if let Some(handler) = self.map.get(&key) {
                return handler.call(req).await;
            }
            Ok(Response::new(
                ErrorCode::ServiceNotFound as u16,
                Body::new_empty(),
            ))
        };
        Box::pin(fut)
    }
}

pub struct FnHandler<'a, State, F, S> {
    f: F,
    state: &'a State,
    _service: std::marker::PhantomData<fn() -> S>,
}

impl<'a, State, F, S> RpcHandler for FnHandler<'a, State, F, S>
where
    State: Send + Sync + 'a,
    F: Handle<'a, State, S> + Send + Sync + 'a,
    S: Service + Send + Sync,
    <S as Service>::Request: FromRequest + Send,
    <S as Service>::Response: ToResponse,
{
    fn call(&self, mut req: Request) -> BoxFuture<'_, Result<Response>> {
        let fut = async move {
            let mut req = S::Request::from_request(&mut req).await?;
            let resp = (self.f).call(self.state, req).await;
            resp.to_response()
        };
        Box::pin(fut)
    }
}

pub trait Handle<'a, State, S: Service> {
    fn call(
        &self,
        state: &'a State,
        req: S::Request,
    ) -> impl Future<Output = S::Response> + Send + 'a;
}

impl<'a, F, Fut, S, State> Handle<'a, State, S> for F
where
    Fut: Future<Output = S::Response> + Send + 'a,
    F: Fn(&'a State, S::Request) -> Fut + Send + Sync + 'a,
    S: Service + Send + Sync,
    S::Request: Send,
    S::Response: Send,
    State: Send + Sync + 'a,
{
    fn call(
        &self,
        state: &'a State,
        req: S::Request,
    ) -> impl Future<Output = S::Response> + Send + 'a {
        (self)(state, req)
    }
}
