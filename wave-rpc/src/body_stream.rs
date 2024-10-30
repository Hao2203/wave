#![allow(unused)]

use bytes::{Buf, BufMut, Bytes};
use futures::{stream::BoxStream, Stream};
use tokio_util::codec::{Decoder, Encoder};

pub struct Body<'a> {
    stream: BoxStream<'a, Chunk>,
}

struct Chunk {
    bytes: Bytes,
}

impl Chunk {
    pub fn new(bytes: Bytes) -> Self {
        Self { bytes }
    }

    pub fn new_empty() -> Self {
        Self::new(Bytes::new())
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn into_bytes(self) -> Bytes {
        self.bytes
    }
}

struct ChunkCodec;

impl ChunkCodec {
    pub const LENTH_SIZE: usize = 8;
}

impl Decoder for ChunkCodec {
    type Item = Chunk;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < ChunkCodec::LENTH_SIZE {
            return Ok(None);
        }

        let len = src.get_u64_le() as usize;
        if src.len() < ChunkCodec::LENTH_SIZE {
            return Ok(None);
        }

        let bytes = src.split_to(len);

        Ok(Some(Chunk::new(bytes.freeze())))
    }
}

impl Encoder<&Chunk> for ChunkCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: &Chunk, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.reserve(item.bytes.len() + ChunkCodec::LENTH_SIZE);
        dst.put_u64_le(item.bytes.len() as u64);
        dst.extend_from_slice(item.bytes.as_ref());
        Ok(())
    }
}

impl Encoder<Chunk> for ChunkCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Chunk, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        self.encode(&item, dst)
    }
}
