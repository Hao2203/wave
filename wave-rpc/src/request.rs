use crate::{
    body_stream::Body,
    error::{Error, Result},
    message::stream::Message,
    service::Version,
    transport::Transport,
    Service,
};
use async_trait::async_trait;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_util::codec::{Decoder, Encoder};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

pub struct Request<'a> {
    pub header: Header,
    pub body: Body<'a>,
}

impl<'a> Request<'a> {
    pub fn new<S>(req: S::Request, service_version: impl Into<Version>) -> Result<Self>
    where
        S: Service,
        S::Request: Message,
    {
        let header = Header {
            service_id: S::ID,
            service_version: service_version.into().into(),
        };
        let body = req.into_body().unwrap();
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

    pub fn body_mut(&mut self) -> &mut Body<'a> {
        &mut self.body
    }

    pub fn into_body(self) -> Body<'a> {
        self.body
    }
}

// impl<'a> Transport for Request<'a> {
//     type Error = crate::error::Error;

//     async fn from_reader(
//         mut io: impl AsyncRead + Send + Sync + Unpin,
//     ) -> Result<Option<Self>, Self::Error>
//     where
//         Self: Sized,
//     {
//         let header = Header::from_reader(&mut io).await?;
//         let req = Body::from_reader(io)
//             .await?
//             .map(|body| Request { header, body });

//         Ok(req)
//     }

//     async fn write_into(
//         &mut self,
//         mut io: impl AsyncWrite + Send + Unpin,
//     ) -> Result<(), Self::Error> {
//         self.header.write_into(&mut io).await?;
//         self.body.write_into(&mut io).await?;
//         Ok(())
//     }
// }

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

    pub async fn from_reader(reader: &mut (impl AsyncRead + Unpin)) -> Result<Self> {
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

#[async_trait]
impl Transport<'_> for Header {
    type Error = crate::error::Error;

    async fn from_reader(
        mut io: impl AsyncRead + Send + Sync + Unpin,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized,
    {
        let header = Header::from_reader(&mut io).await;
        Ok(header.ok())
    }

    async fn write_into(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error> {
        io.write_all(self.as_bytes()).await?;
        Ok(())
    }
}

pub(crate) struct HeaderCodec;

impl Decoder for HeaderCodec {
    type Item = Header;
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

        Ok(Some(header))
    }
}

impl Encoder<Header> for HeaderCodec {
    type Error = Error;

    fn encode(&mut self, item: Header, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let header_bytes = item.as_bytes();
        dst.reserve(header_bytes.len());
        dst.extend_from_slice(header_bytes);
        Ok(())
    }
}

impl Encoder<&Header> for HeaderCodec {
    type Error = Error;

    fn encode(&mut self, item: &Header, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.encode(*item, dst)
    }
}
