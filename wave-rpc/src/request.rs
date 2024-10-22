use crate::{
    body::Body,
    error::{Error, ErrorKind, Result},
};
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio_util::codec::{Decoder, Encoder};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

pub struct Request {
    pub header: Header,
    pub body: Body,
}

impl Request {
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }
}

#[derive(Clone, Copy, TryFromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C, packed)]
pub struct Header {
    pub service_id: u32,
    pub service_version: u32,
}

impl Header {
    pub const SIZE: usize = 8;

    #[inline]
    pub fn buffer() -> [u8; Self::SIZE] {
        [0u8; Self::SIZE]
    }

    pub async fn from_reader(reader: &mut (impl AsyncRead + Unpin)) -> anyhow::Result<Self> {
        let mut header_buf = Header::buffer();

        let _ = reader.read(&mut header_buf).await?;

        let header: Header = Header::try_read_from_bytes(&header_buf[..])
            .map_err(|e| anyhow::anyhow!("Can't parse header from bytes, error: {}", e))?;

        Ok(header)
    }

    pub fn as_bytes(&self) -> &[u8] {
        <Self as IntoBytes>::as_bytes(self)
    }
}

pub struct RequestDecoder<T> {
    codec: T,
}

impl<T> RequestDecoder<T> {
    pub fn new(codec: T) -> Self {
        Self { codec }
    }
}

impl<T> Decoder for RequestDecoder<T>
where
    T: Decoder<Item = Body, Error = Error>,
{
    type Item = Request;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < Header::SIZE {
            return Ok(None);
        }

        let header = {
            let mut header_buf = Header::buffer();
            src.copy_to_slice(&mut header_buf[..]);
            *Header::try_ref_from_bytes(&header_buf[..]).map_err(|e| {
                eprintln!("Can't parse header from bytes, error: {}", e);
                ErrorKind::ParseHeaderFromBytesFailed
            })?
        };

        let body = self.codec.decode(src)?;

        Ok(body.map(|body| Request { header, body }))
    }
}

impl<T, B> Encoder<B> for RequestDecoder<T>
where
    T: Encoder<B, Error = Error>,
{
    type Error = Error;

    fn encode(&mut self, item: B, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.codec.encode(item, dst)
    }
}

pub struct RequestEncoder<T> {
    codec: T,
}

impl<T> RequestEncoder<T> {
    pub fn new(codec: T) -> Self {
        Self { codec }
    }
}

impl<T> Encoder<Request> for RequestEncoder<T>
where
    T: Encoder<Body, Error = Error>,
{
    type Error = Error;

    fn encode(&mut self, item: Request, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let Request { header, body } = item;
        let header_bytes = header.as_bytes();
        dst.reserve(header_bytes.len() + body.len());
        dst.extend_from_slice(header_bytes);
        self.codec.encode(body, dst)?;
        Ok(())
    }
}

impl<T> Decoder for RequestEncoder<T>
where
    T: Decoder<Error = Error>,
{
    type Item = T::Item;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.codec.decode(src)
    }
}
