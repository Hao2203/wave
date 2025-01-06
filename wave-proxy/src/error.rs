#![allow(unused)]
use derive_more::derive::{Display, Error, From};
use std::borrow::Cow;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Display)]
#[display("Inner: {kind}, Message: {message}")]
pub struct Error {
    kind: ErrorInner,
    message: Cow<'static, str>,
}

impl Error {
    pub fn new(kind: impl Into<ErrorInner>, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.kind.source()
    }
}

#[derive(Debug, Display, From, Error)]
#[non_exhaustive]
pub enum ErrorInner {
    #[display("Unexpected error")]
    Unexpected,
    #[display("IO error: {}", _0)]
    #[from]
    IoError(std::io::Error),
    #[display("Get target failed")]
    GetTargetFailed,
    #[display("Proxy failed")]
    ProxyFailed,
    #[display("Unsupported proxy protocol")]
    UnSupportedProxyProtocol,
    #[display("Other error: {}", _0)]
    #[from]
    Other(anyhow::Error),
}

pub(crate) trait Context {
    type Item;
    fn context(self, message: impl Into<Cow<'static, str>>) -> Result<Self::Item>;
}

impl<T, E> Context for Result<T, E>
where
    E: Into<ErrorInner>,
{
    type Item = T;
    fn context(self, message: impl Into<Cow<'static, str>>) -> Result<Self::Item> {
        self.map_err(|e| Error::new(e, message))
    }
}
