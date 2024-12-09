use super::{context::ContextFactory, Result};
use crate::{
    body::Body,
    code::Code,
    error::Error,
    message::{FromBody, IntoBody},
    request::Request,
    response::Response,
    service::Version,
    ServiceDef,
};
use async_trait::async_trait;
use futures_lite::future;
use std::{collections::BTreeMap, future::Future, sync::Arc};

pub struct RpcServiceBuilder<S> {
    map: BTreeMap<ServiceKey, Box<dyn RpcHandler<S> + Send + Sync>>,
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

    // pub fn register<S>(
    //     mut self,
    //     f: impl for<'a> Handle<State, S::Request, Response = S::Response>,
    // ) -> Self
    // where
    //     State: Send + Sync + Clone + 'static,
    //     S: ServiceDef + Send + Sync + 'static,
    //     <S as ServiceDef>::Request: for<'b> FromReader<'b, Error: Into<Error>> + Send + 'static,
    //     <S as ServiceDef>::Response: SendTo<Error: Into<Error>> + Send + Sync + 'static,
    // {
    //     let id = S::ID;
    //     let key = ServiceKey::new(id, self.version);
    //     self.map.insert(
    //         key,
    //         Box::new(FnHandler {
    //             f,
    //             state: self.state.clone(),
    //             _service: std::marker::PhantomData::<fn() -> (S::Request, S::Response)>,
    //         }),
    //     );
    //     self
    // }

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

pub trait Handle<Ctx, Req>: Send + Sync + 'static {
    type Response;
    type Future: Future<Output = Self::Response> + Send;
    fn call(&self, state: &mut Ctx, req: Req) -> Self::Future;
}

impl<F, Fut, Req, Resp, Ctx> Handle<Ctx, Req> for F
where
    F: Fn(&mut Ctx, Req) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Resp> + Send,
    Ctx: Send,
    Req: Send,
    Resp: Send,
{
    type Response = Resp;
    type Future = Fut;

    fn call(&self, state: &mut Ctx, req: Req) -> Self::Future {
        (self)(state, req)
    }
}

#[async_trait]
pub trait RpcHandler<S> {
    async fn call(&self, state: &S, req: Request) -> Result<Response>;
}

struct FnHandler<F, Ctx, Req, Resp> {
    f: F,
    _ctx: std::marker::PhantomData<fn() -> Ctx>,
    _service: std::marker::PhantomData<fn() -> (Req, Resp)>,
}

#[async_trait]
impl<S, Ctx, F, Req, Resp> RpcHandler<S> for FnHandler<F, Ctx, Req, Resp>
where
    S: ContextFactory<Ctx> + Send + std::marker::Sync,
    F: Handle<Ctx, Req, Response = Resp>,
    Req: FromBody<Ctx> + Send,
    Resp: IntoBody + Send,
    Ctx: Send,
{
    async fn call(&self, state: &S, req: Request) -> Result<Response> {
        let mut ctx = state.create_context();
        let body = req.into_body().into_message_body();
        let req = Req::from_body(&mut ctx, body).await.map_err(Into::into)?;
        let resp = self.f.call(&mut ctx, req).await;
        let resp = Response::new(Code::Ok, resp.into_body().into());
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
