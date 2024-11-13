use crate::code::Code;
use bytes::Bytes;
use derive_more::derive::Display;
use std::fmt::{Debug, Display};

pub type Result<T, E = Error> = core::result::Result<T, E>;
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, Display, derive_more::Error)]
pub struct Error(#[error(not(source))] Box<dyn RpcError + Send + Sync>);

pub trait RpcError: Display + Debug {
    fn code(&self) -> Code;

    fn payload(&self) -> Bytes;
}
