use super::*;

#[derive(Debug, thiserror::Error, Serialize)]
#[error("{:?}", self)]
pub enum Error {
    // mod content
    TextLengthBiggerThan1024,
}

pub type Result<T> = core::result::Result<T, Error>;
