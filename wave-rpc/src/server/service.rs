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
use futures::future::{ready, BoxFuture, Ready};
use std::{collections::BTreeMap, future::Future, ops::AsyncFn, sync::Arc};

type Response = crate::response::Response<Body<'static>>;

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
        f: impl for<'a> Handle<State, S::Request, Response = S::Response>,
    ) -> Self
    where
        State: Send + Sync + Clone + 'static,
        S: ServiceDef + Send + Sync + 'static,
        <S as ServiceDef>::Request: for<'b> FromReader<'b, Error: Into<Error>> + Send + 'static,
        <S as ServiceDef>::Response: SendTo<Error: Into<Error>> + Send + Sync + 'static,
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

impl Service<Request> for RpcService {
    type Response = Response;
    type Error = Error;
    fn call(
        &self,
        mut req: Request,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send + 'static {
        let id = req.header.service_id;
        let version = req.header.service_version;
        let key = ServiceKey::new(id, version);
        let handler = self.map.get(&key).expect("service not found");
        handler.call(req)
    }
}

pub trait Handle<State, Req>: Clone + Send + Sync + 'static {
    type Response;
    type Future: Future<Output = Self::Response> + Send;
    fn call(self, state: State, req: Req) -> Self::Future;
}

impl<F, Fut, Req, Resp, State> Handle<State, Req> for F
where
    F: FnOnce(State, Req) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Resp> + Send,
    State: Send,
    Req: Send,
    Resp: Send,
{
    type Response = Resp;
    type Future = Fut;

    fn call(self, state: State, req: Req) -> Self::Future {
        (self)(state, req)
    }
}

pub trait RpcHandler {
    fn call(&self, req: Request) -> BoxFuture<'static, Result<Response>>;
}

struct FnHandler<State, F, Req, Resp> {
    f: F,
    state: State,
    _service: std::marker::PhantomData<fn() -> (Req, Resp)>,
}

impl<State, F, Req, Resp> RpcHandler for FnHandler<State, F, Req, Resp>
where
    State: Clone + Sync + Send + 'static,
    F: Handle<State, Req, Response = Resp>,
    for<'a> Req: FromReader<'a, Error: Into<Error>> + Send,
    Resp: SendTo<Error: Into<Error>> + Send + Sync + 'static,
{
    fn call(&self, mut req: Request) -> BoxFuture<'static, Result<Response>> {
        let f = self.f.clone();
        let state = self.state.clone();
        let fut = async move {
            let req = Req::from_reader(req.body_mut()).await.map_err(Into::into);
            match req {
                Ok(req) => {
                    let resp = f.call(state, req).await;
                    let body = Body::new(resp);
                    Ok(Response::success(body))
                }
                Err(err) => Ok(Response::from_error(err)),
            }
        };
        Box::pin(fut)
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
