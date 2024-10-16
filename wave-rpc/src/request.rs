use crate::body::{Body, BodyType};
use tokio::io::{AsyncRead, AsyncReadExt};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

pub struct Request<'conn> {
    pub header: &'conn Header,
    pub body: Body<'conn>,
}

impl<'conn> Request<'conn> {
    pub fn header(&self) -> &Header {
        self.header
    }

    pub fn body(&self) -> &Body<'conn> {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut Body<'conn> {
        &mut self.body
    }
}

#[derive(Clone, Copy, TryFromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C, packed)]
pub struct Header {
    pub service_id: u64,
    pub body_type: BodyType,
    pub body_size: u64, // if body_type == BodyType::Bytes then this is the size in bytes else it's the stream item length
}

impl Header {
    pub const SIZE: usize = 17;

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

    pub fn body_size(&self) -> usize {
        self.body_size as usize
    }

    pub fn as_bytes(&self) -> &[u8] {
        <Self as IntoBytes>::as_bytes(self)
    }
}
