#![allow(unused)]
use crate::{error::Result, service::Version};
use async_compat::CompatExt;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures_lite::{AsyncRead, AsyncReadExt as _, Stream, StreamExt};
use std::pin::Pin;
use tokio_util::codec::{Decoder, Encoder, FramedRead};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

pub struct Request {
    pub header: Header,
    pub body: Pin<Box<dyn Stream<Item = Bytes> + Send>>,
}

impl Request {
    // pub fn new(header: Header, body: Body) -> Self {
    //     Self { header, body }
    // }

    pub async fn from_reader(
        mut reader: impl AsyncRead + Unpin + Send + 'static,
    ) -> Result<Request> {
        let header = Header::from_reader(&mut reader).await?;
        let stream = FramedRead::new(reader.compat(), FrameCodec);
        let body = stream
            .filter_map(|frame| match frame {
                Ok(ReqFrame::Data(data)) => Some(data),
                _ => None,
            })
            .boxed();
        let request = Request { header, body };

        Ok(request)
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
}

impl Stream for Request {
    type Item = Bytes;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.body.as_mut().poll_next(cx)
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
    const BUFFER: [u8; Self::SIZE] = [0u8; Self::SIZE];

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        <Self as IntoBytes>::as_bytes(self)
    }

    pub(crate) async fn from_reader(reader: &mut (impl AsyncRead + Unpin)) -> Result<Self> {
        let mut buf = Self::BUFFER;
        reader.read_exact(&mut buf).await?;
        Ok(Header::try_read_from_bytes(&buf)?)
    }
}

#[derive(Debug, Clone)]
pub enum ReqFrame {
    End,
    Data(Bytes),
}

impl ReqFrame {
    const SIZE_LEN: usize = 4;
    const EOS_LEN: usize = 1;

    pub(crate) fn codec() -> FrameCodec {
        FrameCodec
    }

    pub fn new(data: Bytes) -> ReqFrame {
        ReqFrame::Data(data)
    }

    pub fn new_empty() -> ReqFrame {
        Self::new(Bytes::new())
    }

    pub fn is_end_of_stream(&self) -> bool {
        matches!(self, ReqFrame::End)
    }
}

pub(crate) struct FrameCodec;

impl Encoder<ReqFrame> for FrameCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: ReqFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(ReqFrame::EOS_LEN);
        match item {
            ReqFrame::End => dst.put_u8(0),
            ReqFrame::Data(data) => {
                let data_size = data.len() as u32;
                dst.put_u8(1);
                dst.reserve(ReqFrame::SIZE_LEN + data_size as usize);
                dst.put_u32(data_size);
                dst.put(data);
            }
        };
        Ok(())
    }
}

impl Decoder for FrameCodec {
    type Error = std::io::Error;
    type Item = ReqFrame;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<ReqFrame>, Self::Error> {
        if src.len() < ReqFrame::EOS_LEN {
            return Ok(None);
        }

        let is_end_of_stream = src.get_u8() != 0;
        if is_end_of_stream {
            return Ok(Some(ReqFrame::End));
        }

        if src.len() < ReqFrame::SIZE_LEN {
            return Ok(None);
        }
        let data_size = src.get_u32();

        if src.len() < data_size as usize {
            return Ok(None);
        }

        let data = src.split_to(data_size as usize).freeze();
        let frame = ReqFrame::Data(data);

        Ok(Some(frame))
    }
}
