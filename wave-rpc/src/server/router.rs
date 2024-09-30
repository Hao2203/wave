#![allow(unused)]
use super::Handle;
use crate::service::Service;
use std::{collections::HashMap, future::Future};

pub trait Router<'conn, Conn> {
    type Handler: Handle<'conn, Conn>;

    fn route(&self, conn: &'conn mut Conn) -> impl Future<Output = anyhow::Result<Self::Handler>>;

    fn register<S, Req>(&mut self, service: &S)
    where
        S: Service<Req> + Send + Sync + 'static,
        Req: Send + 'static,
        S::Response: Send + 'static;
}
