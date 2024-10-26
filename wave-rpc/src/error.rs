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
pub enum Code {
    ServiceNotFound,
    CodecError,
    IoError,
    BodyTooLarge,
    Other(u16),
}

impl From<Code> for u16 {
    fn from(val: Code) -> Self {
        match val {
            Code::ServiceNotFound => 1,
            Code::CodecError => 2,
            Code::IoError => 3,
            Code::BodyTooLarge => 4,
            Code::Other(code) => code,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{:?}", self)]
pub struct ParseCodeError(pub u16);

impl TryFrom<u16> for Code {
    type Error = ParseCodeError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::ServiceNotFound),
            2 => Ok(Self::CodecError),
            3 => Ok(Self::IoError),
            4 => Ok(Self::BodyTooLarge),
            _ => Err(ParseCodeError(value)),
        }
    }
}

impl TryFrom<Error> for Code {
    type Error = Error;
    fn try_from(val: Error) -> Result<Code, Self::Error> {
        match val {
            Error::ServiceNotFound => Ok(Code::ServiceNotFound),
            Error::Bincode(_) | Error::ParseHeaderFromBytesFailed => Ok(Code::CodecError),
            Error::Io(_) => Err(val),
            Error::BodyTooLarge => Ok(Code::BodyTooLarge),
        }
    }
}
