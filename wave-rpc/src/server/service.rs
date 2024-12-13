use super::Result;
use crate::{
    body::Body,
    code::Code,
    error::Error,
    message::{FromStream, IntoStream},
    request::Request,
    response::Response,
    service::Version,
    ServiceDef,
};
use async_trait::async_trait;
use futures_lite::future;
use std::{collections::BTreeMap, future::Future, sync::Arc};

pub struct RpcServiceBuilder<S> {
    map: BTreeMap<ServiceKey, Box<dyn RpcHandler + Send + Sync>>,
    state: Arc<S>,
    version: Version,
}

impl RpcServiceBuilder<()> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            state: ().into(),
            version: Default::default(),
        }
    }
}
impl<State> RpcServiceBuilder<State> {
    pub fn with_state(state: Arc<State>) -> Self {
        Self {
            map: BTreeMap::new(),
            state,
            version: Default::default(),
        }
    }

    // pub fn set_state<State2>(self, state: State2) -> RpcServiceBuilder<State2> {
    //     RpcServiceBuilder {
    //         map: self.map,
    //         state,
    //         version: self.version,
    //     }
    // }

    pub fn version(mut self, version: impl Into<Version>) -> Self {
        self.version = version.into();
        self
    }

    pub fn register<S>(
        mut self,
        f: impl Handle<Arc<State>, S::Request, Response = S::Response>,
    ) -> Self
    where
        State: Send + Sync + 'static,
        S: ServiceDef,
        <S as ServiceDef>::Request: FromStream + Send + 'static,
        <S as ServiceDef>::Response: IntoStream + Send + Sync + 'static,
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

    // pub fn merge(mut self, other: RpcService) -> Self {
    //     self.map.extend(other.map);
    //     self
    // }

    // pub fn build(self) -> RpcService {
    //     RpcService { map: self.map }
    // }
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

// pub struct RpcService {
//     map: BTreeMap<ServiceKey, Box<dyn RpcHandler + Send + Sync>>,
// }

pub trait Handle<State, Req>: Send + Sync + 'static {
    type Response;
    type Future: Future<Output = Self::Response> + Send;
    fn call(&self, state: State, req: Req) -> Self::Future;
}

impl<F, Fut, Req, Resp, State> Handle<State, Req> for F
where
    F: Fn(State, Req) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Resp> + Send,
    State: Send,
    Req: Send,
    Resp: Send,
{
    type Response = Resp;
    type Future = Fut;

    fn call(&self, state: State, req: Req) -> Self::Future {
        (self)(state, req)
    }
}

#[async_trait]
pub trait RpcHandler {
    async fn call(&self, req: Request) -> Result<Response>;
}

struct FnHandler<F, S, Req, Resp> {
    f: F,
    state: Arc<S>,
    _service: std::marker::PhantomData<fn() -> (Req, Resp)>,
}

#[async_trait]
impl<S, F, Req, Resp> RpcHandler for FnHandler<F, S, Req, Resp>
where
    S: Send + Sync,
    F: Handle<Arc<S>, Req, Response = Resp>,
    Req: FromStream + Send,
    Resp: IntoStream + Send,
{
    async fn call(&self, req: Request) -> Result<Response> {
        let state = self.state.clone();
        let req = Req::from_stream(req).await.unwrap();
        let resp = self.f.call(state, req).await;
        let resp = Response::new(Code::Ok, todo!());
        Ok(resp)
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
