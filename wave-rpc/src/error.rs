pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "bincode")]
    #[error(transparent)]
    Bincode(#[from] bincode::Error),

    #[cfg(feature = "rmp")]
    #[error(transparent)]
    RmpEncode(#[from] rmp_serde::encode::Error),

    #[cfg(feature = "rmp")]
    #[error(transparent)]
    RmpDeocde(#[from] rmp_serde::decode::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("body too large")]
    BodyTooLarge,

    #[error("parse header from bytes failed")]
    ParseHeaderFromBytesFailed,

    #[error("parse error code failed")]
    ParseErrorCodeFailed,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Code {
    ServiceNotFound = 1,
}

impl TryFrom<u16> for Code {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::ServiceNotFound),
            _ => Err(Error::ParseErrorCodeFailed),
        }
    }
}

impl From<Code> for u16 {
    fn from(val: Code) -> Self {
        val as u16
    }
}
