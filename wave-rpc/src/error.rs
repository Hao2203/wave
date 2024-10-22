use std::sync::Arc;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] Arc<ErrorKind>);

impl<T: Into<ErrorKind>> From<T> for Error {
    fn from(kind: T) -> Self {
        Self(Arc::new(kind.into()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[cfg(feature = "bincode")]
    #[error(transparent)]
    Bincode(#[from] bincode::Error),
}
