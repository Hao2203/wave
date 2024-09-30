#![allow(unused)]

pub use crate::error::Result;
use crate::{
    author::CurrentAuthor,
    message::Message,
    session::{actor::Actor, stats::SessionStats, SessionCreator, SessionHandle},
};
use futures::Stream;
use std::future::Future;

pub trait SessionService {
    fn list_sessions(
        &self,
    ) -> impl Future<Output = Result<impl Stream<Item = SessionStats>>> + Send;

    fn get_session(
        &self,
        index: &impl SessionHandle,
    ) -> impl Future<Output = Result<SessionStats>> + Send;

    fn create_session(
        &self,
        creator: SessionCreator,
    ) -> impl Future<Output = Result<SessionStats>> + Send;

    fn enter_session(
        &self,
        index: &impl SessionHandle,
        author: &impl CurrentAuthor,
    ) -> impl Future<Output = Result<Actor>> + Send;
}

pub struct Service<'node> {
    node: &'node iroh::node::FsNode,
}

impl<'node> Service<'node> {
    pub async fn new(node: &'node iroh::node::FsNode) -> Self {
        Self { node }
    }
}
