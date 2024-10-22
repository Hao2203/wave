use crate::error::{Error, ErrorKind, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use tokio_util::codec::{Decoder, Encoder};

#[non_exhaustive]
pub struct Body {
    data: Bytes,
}

impl Body {
    pub const LENTH_SIZE: usize = 8;

    pub fn new(data: Bytes) -> Self {
        Self { data }
    }

    pub fn new_empty() -> Self {
        Self::new(Bytes::new())
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn as_slice(&self) -> &[u8] {
        self.data.as_ref()
    }

    pub fn into_bytes(self) -> Bytes {
        self.data
    }
}

#[cfg(feature = "bincode")]
impl Body {
    pub fn bincode_encode(data: impl Serialize) -> Result<Self> {
        let bytes = bincode::serialize(&data)?;
        Ok(Self::new(bytes.into()))
    }

    pub fn bincode_decode<'a, T: Deserialize<'a>>(&'a self) -> Result<T> {
        let bytes = self.as_slice();
        let value = bincode::deserialize(bytes)?;
        Ok(value)
    }
}

#[cfg(feature = "rmp")]
impl Body {
    pub fn rmp_encode(data: impl Serialize) -> Result<Self> {
        let bytes = rmp_serde::to_vec(&data)?;
        Ok(Self::new(bytes.into()))
    }

    pub fn rmp_decode<'a, T: Deserialize<'a>>(&'a self) -> Result<T> {
        let bytes = self.as_slice();
        let value = rmp_serde::from_slice(bytes)?;
        Ok(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BodyCodec {
    max_size: usize,
}

impl BodyCodec {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

impl Encoder<Body> for BodyCodec {
    type Error = Error;

    fn encode(&mut self, item: Body, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if item.len() > self.max_size {
            return Err(ErrorKind::BodyTooLarge)?;
        }
        dst.reserve(item.len() + Body::LENTH_SIZE); // 8 bytes for length
        dst.put_u64_le(item.len() as u64);
        dst.extend_from_slice(item.as_slice());
        Ok(())
    }
}

impl Decoder for BodyCodec {
    type Item = Body;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // 8 bytes for length
        if src.len() < Body::LENTH_SIZE {
            return Ok(None);
        }
        let length = src.get_u64_le();

        if src.len() < length as usize + Body::LENTH_SIZE {
            return Ok(None);
        }
        let data = src.split_to(length as usize);

        Ok(Some(Body { data: data.into() }))
    }
}
