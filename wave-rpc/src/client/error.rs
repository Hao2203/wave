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
}

pub type Result<T, E = ClientError> = std::result::Result<T, E>;
