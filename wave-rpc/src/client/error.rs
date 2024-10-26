use crate::{error::ErrorCode, Request};
use anyhow::anyhow;
use deadpool::managed::{BuildError, PoolError};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error(transparent)]
    Base(#[from] crate::error::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("receive response failed")]
    ReceiveResponseFailed,

    #[error("service not found, id = {id}, version = {version}")]
    ServiceNotFound { id: u32, version: u32 },

    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[error(transparent)]
    PoolBuildError(#[from] BuildError),

    #[error("parse error code failed")]
    ParseErrorCodeFailed,
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

impl From<(ErrorCode, &Request)> for ClientError {
    fn from((code, req): (ErrorCode, &Request)) -> Self {
        match code {
            ErrorCode::ServiceNotFound => ClientError::ServiceNotFound {
                id: req.header.service_id,
                version: req.header.service_version,
            },
            _ => ClientError::Other(anyhow!("unknown error code: {:?}", code)),
        }
    }
}
