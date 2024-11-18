use super::Result;
use crate::{
    body::Body,
    code::Code,
    error::Error,
    message::{FromReader, SendTo},
    request::RequestReader as Request,
    service::{Service, Version},
    ServiceDef,
};
use async_trait::async_trait;
use std::{collections::BTreeMap, future::Future, ops::AsyncFn, sync::Arc};

type Response<'a> = crate::response::Response<Body<'a>>;

pub struct RpcServiceBuilder<S> {
    map: BTreeMap<ServiceKey, Box<dyn RpcHandler + Send + Sync>>,
    state: S,
    version: Version,
}

impl RpcServiceBuilder<()> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            state: (),
            version: Default::default(),
        }
    }
}
impl<State> RpcServiceBuilder<State> {
    pub fn with_state(state: State) -> Self {
        Self {
            map: BTreeMap::new(),
            state,
            version: Default::default(),
        }
    }

    pub fn set_state<State2>(self, state: State2) -> RpcServiceBuilder<State2> {
        RpcServiceBuilder {
            map: self.map,
            state,
            version: self.version,
        }
    }

    pub fn version(mut self, version: impl Into<Version>) -> Self {
        self.version = version.into();
        self
    }

    pub fn register<S>(
        mut self,
        f: impl for<'a> Handle<&'a State, S::Request, Response = S::Response> + Send + Sync + 'static,
    ) -> Self
    where
        State: Send + Sync + Clone + 'static,
        S: ServiceDef + Send + Sync + 'static,
        <S as ServiceDef>::Request: for<'b> FromReader<'b> + Send,
        <S as ServiceDef>::Response: SendTo<Error: Into<Error>> + Send,
    {
        let id = S::ID;
        let key = ServiceKey::new(id, self.version);
        self.map.insert(
            key,
            Box::new(FnHandler {
                f,
                state: self.state.clone(),
                _service: std::marker::PhantomData::<fn() -> (S::Request, S::Response)>,
            }),
        );
        self
    }

    pub fn merge(mut self, other: RpcService) -> Self {
        self.map.extend(other.map);
        self
    }

    pub fn build(self) -> RpcService {
        RpcService { map: self.map }
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

pub struct RpcService {
    map: BTreeMap<ServiceKey, Box<dyn RpcHandler + Send + Sync>>,
}

impl Service<Request<'static>> for Arc<RpcService> {
    type Response = Response<'static>;
    type Error = Error;
    fn call(
        &self,
        mut req: Request<'static>,
    ) -> impl Future<Output = std::result::Result<Self::Response, Self::Error>> + Send + 'static
    {
        let id = req.header.service_id;
        let version = req.header.service_version;
        let key = ServiceKey::new(id, version);
        let arc_self = self.clone();
        let fut = async move {
            if let Some(handler) = arc_self.map.get(&key) {
                return handler.call(&mut req).await;
            }
            todo!()
        };
        fut
    }
}

// #[async_trait]
// impl<'a, State> RpcHandler for RpcService<'a, State>
// where
//     State: Sync + 'a,
// {
//     async fn call(&self, req: &mut Request) -> Result<Response> {
//         let id = req.header.service_id;
//         let version = req.header.service_version;
//         let key = ServiceKey::new(id, version);
//         if let Some(handler) = self.map.get(&key) {
//             return handler.call(req).await;
//         }
//         Ok(Response::new(Code::ServiceNotFound, Body::new_empty()))
//     }
// }

pub trait Handle<State, Req> {
    type Response;
    fn call(&self, state: State, req: Req) -> impl Future<Output = Self::Response> + Send;
}

impl<F, Req, Resp, State> Handle<State, Req> for F
where
    F: AsyncFn(State, Req) -> Resp + Sync,
    for<'a> F::CallRefFuture<'a>: Send,
    State: Send,
    Req: Send,
    Resp: Send,
{
    type Response = Resp;
    async fn call(&self, state: State, req: Req) -> Resp {
        (self)(state, req).await
    }
}

#[async_trait]
pub trait RpcHandler {
    async fn call(&self, req: &mut Request) -> Result<Response>;
}

struct FnHandler<State, F, Req, Resp> {
    f: F,
    state: State,
    _service: std::marker::PhantomData<fn() -> (Req, Resp)>,
}

#[async_trait]
impl<State, F, Req, Resp> RpcHandler for FnHandler<State, F, Req, Resp>
where
    State: Sync,
    F: for<'a> Handle<&'a State, Req, Response = Resp> + Sync,
    Req: for<'a> FromReader<'a> + Send,
    Resp: SendTo<Error: Into<Error>> + Send,
{
    async fn call(&self, req: &mut Request) -> Result<Response> {
        let req = Req::from_reader(req.body_mut()).await.unwrap();
        let resp = (self.f).call(&self.state, req).await;
        let body = Body::new(resp);
        Ok(Response::success(body))
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        is_send::<RpcServiceBuilder<()>>();
    }

    fn is_send<T: Send>() {}
}
