use crate::{
    body::Body,
    error::{Error, Result},
    message::Message,
    service::Version,
    Service,
};
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio_util::codec::{Decoder, Encoder};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

#[derive(Debug, Clone)]
pub struct Request {
    pub header: Header,
    pub body: Body,
}

impl Request {
    pub fn new<S>(
        req: <S::Request as Message>::Inner,
        service_version: impl Into<Version>,
    ) -> Result<Self>
    where
        S: Service,
        S::Request: Message,
    {
        let header = Header {
            service_id: S::ID,
            service_version: service_version.into().into(),
        };
        let body = S::Request::into_body(req).unwrap();
        Ok(Self { header, body })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn service_id(&self) -> u32 {
        self.header.service_id
    }

    pub fn service_version(&self) -> Version {
        Version::from(self.header.service_version)
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }

    pub fn into_body(self) -> Body {
        self.body
    }
}

#[derive(Debug, Clone, Copy, TryFromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
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

pub(crate) struct RequestDecoder<T> {
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
                Error::ParseHeaderFromBytesFailed
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

pub(crate) struct RequestEncoder<T> {
    codec: T,
}

impl<T> RequestEncoder<T> {
    pub fn new(codec: T) -> Self {
        Self { codec }
    }
}

impl<T> Encoder<Request> for RequestEncoder<T>
where
    T: for<'a> Encoder<&'a Body, Error = Error>,
{
    type Error = Error;

    fn encode(&mut self, item: Request, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.encode(&item, dst)
    }
}

impl<T> Encoder<&Request> for RequestEncoder<T>
where
    T: for<'a> Encoder<&'a Body, Error = Error>,
{
    type Error = Error;

    fn encode(&mut self, item: &Request, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let Request { header, body } = item;
        let header_bytes = header.as_bytes();
        dst.reserve(header_bytes.len() + body.len());
        dst.extend_from_slice(header_bytes);
        self.codec.encode(body, dst)?;
        Ok(())
    }
}

impl<T> Encoder<Body> for RequestEncoder<T>
where
    T: for<'a> Encoder<&'a Body, Error = Error>,
{
    type Error = Error;

    fn encode(&mut self, item: Body, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.codec.encode(&item, dst)
    }
}

impl<T> Encoder<&Body> for RequestEncoder<T>
where
    T: for<'a> Encoder<&'a Body, Error = Error>,
{
    type Error = Error;

    fn encode(&mut self, item: &Body, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.codec.encode(item, dst)
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
