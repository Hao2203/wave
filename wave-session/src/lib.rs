#![allow(unused)]

pub use session::Session;
use session::SessionId;
use std::future::Future;
use wave_core::EntityStore;

pub mod session;

pub trait SessionStore: EntityStore<Session> {}
