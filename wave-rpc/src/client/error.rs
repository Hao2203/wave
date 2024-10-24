use deadpool::managed::{BuildError, PoolError};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error(transparent)]
    Base(#[from] crate::error::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("receive response failed")]
    ReceiveResponseFailed,

    #[error("error code: {0}")]
    ErrorWithCode(u16),

    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[error(transparent)]
    PoolBuildError(#[from] BuildError),
}

pub type Result<T, E = ClientError> = std::result::Result<T, E>;

impl From<PoolError<Self>> for ClientError {
    fn from(e: PoolError<Self>) -> Self {
        match e {
            PoolError::Backend(e) => e,
            _ => ClientError::Other(e.into()),
        }
    }
}
