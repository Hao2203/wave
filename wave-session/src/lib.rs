#![allow(unused)]

pub use session::Session;
use session::SessionId;
use std::future::Future;

pub mod record;
pub mod session;

pub trait SessionStore {}
