use derive_more::derive::Display;

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

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    #[display("Unexpected error")]
    Unexpected,
    #[display("IO error: {}", _0)]
    IoError(std::io::ErrorKind),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::with_source(ErrorKind::IoError(e.kind()), e)
    }
}
