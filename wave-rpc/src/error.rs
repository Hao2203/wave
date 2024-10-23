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
}
