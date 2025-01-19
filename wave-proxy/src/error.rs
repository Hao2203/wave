#![allow(unused)]
use derive_more::derive::{Display, Error, From};
use std::fmt::{Debug, Display};

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, From, Display)]
#[display("Kind: {kind}, source: {source:?}")]
pub struct Error {
    kind: ErrorKind,
    source: Option<anyhow::Error>,
}

impl Error {
    pub fn new(kind: ErrorKind, source: Option<impl Into<anyhow::Error>>) -> Self {
        Self {
            kind,
            source: source.map(Into::into),
        }
    }

    pub fn message(kind: ErrorKind, message: impl Display + Debug + Sync + Send + 'static) -> Self {
        Self {
            kind,
            source: Some(anyhow::Error::msg(message)),
        }
    }

    pub fn with_kind(kind: ErrorKind) -> Self {
        Self { kind, source: None }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.kind.source()
    }
}

#[derive(Debug, Clone, Copy, Display, Error)]
#[non_exhaustive]
pub enum ErrorKind {
    #[display("Unexpected error")]
    Unexpected,
    #[display("IO error")]
    IoError,
    #[display("Get target failed")]
    GetTargetFailed,
    #[display("Proxy failed")]
    ProxyFailed,
    #[display("Unsupported proxy protocol")]
    UnSupportedProxyProtocol,
    #[display("Timeout")]
    Timeout,
    #[display("Other error")]
    Other,
}

pub(crate) trait WithKind {
    type Item;
    fn with_kind(self, kind: ErrorKind) -> Result<Self::Item>;
}

impl<T, E> WithKind for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    type Item = T;
    fn with_kind(self, kind: ErrorKind) -> Result<Self::Item> {
        self.map_err(|e| Error::new(kind, Some(e)))
    }
}

impl<T> WithKind for Option<T> {
    type Item = T;
    fn with_kind(self, kind: ErrorKind) -> Result<Self::Item> {
        self.ok_or(Error::with_kind(kind))
    }
}
