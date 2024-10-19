use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[non_exhaustive]
pub struct Body {
    data: Bytes,
}

impl Body {
    pub const MAX_LEN: usize = u64::MAX as usize;

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

pub struct BodyCodec {
    max_size: usize,
}

impl BodyCodec {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

impl Encoder<Body> for BodyCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Body, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if item.len() > self.max_size {
            return Err(anyhow::anyhow!("body too large"));
        }
        dst.reserve(item.len() + 8); // 8 bytes for length
        dst.put_u64_le(item.len() as u64);
        dst.extend_from_slice(item.as_slice());
        Ok(())
    }
}

impl Decoder for BodyCodec {
    type Item = Body;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // 8 bytes for length
        if src.len() < 8 {
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
