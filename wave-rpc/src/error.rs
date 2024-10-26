pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "bincode")]
    #[error(transparent)]
    Bincode(#[from] bincode::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("body too large")]
    BodyTooLarge,

    #[error("parse header from bytes failed")]
    ParseHeaderFromBytesFailed,

    #[error("service not found")]
    ServiceNotFound,
}

#[derive(Debug)]
pub enum ErrorCode {
    ServiceNotFound,
    CodecError,
    IoError,
    BodyTooLarge,
    Other(u16),
}

impl From<ErrorCode> for u16 {
    fn from(val: ErrorCode) -> Self {
        match val {
            ErrorCode::ServiceNotFound => 1,
            ErrorCode::CodecError => 2,
            ErrorCode::IoError => 3,
            ErrorCode::BodyTooLarge => 4,
            ErrorCode::Other(code) => code,
        }
    }
}

impl From<u16> for ErrorCode {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::ServiceNotFound,
            2 => Self::CodecError,
            3 => Self::IoError,
            4 => Self::BodyTooLarge,
            _ => Self::Other(value),
        }
    }
}

impl TryFrom<Error> for ErrorCode {
    type Error = Error;
    fn try_from(val: Error) -> Result<ErrorCode, Self::Error> {
        match val {
            Error::ServiceNotFound => Ok(ErrorCode::ServiceNotFound),
            Error::Bincode(_) | Error::ParseHeaderFromBytesFailed => Ok(ErrorCode::CodecError),
            Error::Io(_) => Err(val),
            Error::BodyTooLarge => Ok(ErrorCode::BodyTooLarge),
        }
    }
}
