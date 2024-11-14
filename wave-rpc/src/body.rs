#![allow(unused)]
use crate::error::{Error, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone)]
pub struct Body {
    data: Bytes,
}

impl Body {
    pub const LENTH_SIZE: usize = 8;

    #[inline]
    pub const fn new(data: Bytes) -> Self {
        Self { data }
    }

    #[inline]
    pub const fn new_empty() -> Self {
        Self::new(Bytes::new())
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.data.as_ref()
    }

    #[inline]
    pub fn into_bytes(self) -> Bytes {
        self.data
    }

    #[inline]
    pub const fn bytes_mut(&mut self) -> &mut Bytes {
        &mut self.data
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BodyCodec {
    max_size: usize,
}

impl BodyCodec {
    #[inline]
    pub const fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

impl Encoder<Body> for BodyCodec {
    type Error = std::io::Error;

    #[inline]
    fn encode(&mut self, item: Body, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.encode(&item, dst)
    }
}

impl Encoder<&Body> for BodyCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: &Body, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if item.len() > self.max_size {
            return Err(todo!());
        }
        dst.reserve(item.len() + Body::LENTH_SIZE); // 8 bytes for length
        dst.put_u64_le(item.len() as u64);
        dst.extend_from_slice(item.as_slice());
        Ok(())
    }
}

impl Encoder<&mut Body> for BodyCodec {
    type Error = std::io::Error;

    #[inline]
    fn encode(&mut self, item: &mut Body, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.encode(item as &Body, dst)
    }
}

impl Decoder for BodyCodec {
    type Item = Body;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // 8 bytes for length
        if src.len() < Body::LENTH_SIZE {
            return Ok(None);
        }
        let length = src.get_u64_le();

        if src.len() < length as usize {
            return Ok(None);
        }
        let data = src.split_to(length as usize);

        Ok(Some(Body { data: data.into() }))
    }
}
