use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[non_exhaustive]
pub struct Body {
    length: u32,
    data: Bytes,
}

impl Body {
    pub fn new(data: Bytes) -> Self {
        Self {
            length: data.len() as u32,
            data,
        }
    }
    pub fn length(&self) -> usize {
        self.length as usize
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
        if item.length() > self.max_size {
            return Err(anyhow::anyhow!("body too large"));
        }
        dst.reserve(item.length() + 4);
        dst.put_u32_le(item.length() as u32);
        dst.extend_from_slice(item.as_slice());
        Ok(())
    }
}

impl Decoder for BodyCodec {
    type Item = Body;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }
        let length = src.get_u32_le();
        if src.len() < length as usize {
            return Ok(None);
        }
        let data = src.split_to(length as usize);
        Ok(Some(Body {
            length,
            data: data.into(),
        }))
    }
}
