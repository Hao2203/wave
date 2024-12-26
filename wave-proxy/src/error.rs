use derive_more::derive::Display;
use std::fmt::Display;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Display)]
#[display("Kind: {kind}, Source: {source:?}")]
pub struct Error {
    kind: ErrorKind,
    source: Option<anyhow::Error>,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, source: None }
    }

    pub fn with_source(kind: ErrorKind, source: impl Into<anyhow::Error>) -> Self {
        Self {
            kind,
            source: Some(source.into()),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    Unexpected,
}

impl ErrorKind {
    pub fn into_str(self) -> &'static str {
        self.into()
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.into_str())
    }
}

impl From<ErrorKind> for &'static str {
    fn from(kind: ErrorKind) -> Self {
        match kind {
            ErrorKind::Unexpected => "Unexpected",
        }
    }
}
