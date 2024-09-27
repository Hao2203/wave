use std::sync::Arc;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] Arc<ErrorKind>);

impl<T: Into<ErrorKind>> From<T> for Error {
    fn from(kind: T) -> Self {
        Self(Arc::new(kind.into()))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{:?}", self)]
pub enum ErrorKind {
    SessionNameTooLong,
    AuthorNameTooLong,
    AuthorNotFound,
    Other(#[from] anyhow::Error),
}
