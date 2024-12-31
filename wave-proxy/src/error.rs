#![allow(unused)]
use derive_more::derive::{Display, From};
use std::borrow::Cow;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Display)]
#[display("Kind: {kind}, Message: {message}, Source: {source:?}")]
pub struct Error {
    kind: ErrorKind,
    message: Cow<'static, str>,
    source: Option<anyhow::Error>,
}

impl Error {
    pub fn new(kind: impl Into<ErrorKind>, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
            source: None,
        }
    }

    pub fn set_source(mut self, source: impl Into<anyhow::Error>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }
}

#[derive(Debug, Display, From, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    #[display("Unexpected error")]
    Unexpected,
    #[display("IO error: {}", _0)]
    #[from]
    IoError(std::io::ErrorKind),
    #[display("Get target failed")]
    GetTargetFailed,
    #[display("Proxy failed")]
    ProxyFailed,
    #[display("Unsupported proxy protocol")]
    UnSupportedProxyProtocol,
}

impl From<&std::io::Error> for ErrorKind {
    fn from(value: &std::io::Error) -> Self {
        ErrorKind::IoError(value.kind())
    }
}

pub(crate) trait Context {
    type Item;
    fn context(self, message: impl Into<Cow<'static, str>>) -> Result<Self::Item>;
}

impl<T, E> Context for Result<T, E>
where
    E: Into<anyhow::Error>,
    for<'a> &'a E: Into<ErrorKind>,
{
    type Item = T;
    fn context(self, message: impl Into<Cow<'static, str>>) -> Result<Self::Item> {
        self.map_err(|e| Error::new(&e, message).set_source(e))
    }
}
