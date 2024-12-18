use crate::error::Result;
use bytes::Bytes;

pub trait Message {
    fn into_bytes(self) -> Result<Bytes>;
    fn from_bytes(bytes: Bytes) -> Result<Self>
    where
        Self: Sized;
}
