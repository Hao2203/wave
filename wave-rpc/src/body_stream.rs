#![allow(unused)]
use crate::message::Message;
use async_stream::stream;
use async_trait::async_trait;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{
    stream::{self, BoxStream},
    AsyncRead, AsyncReadExt, AsyncWrite, SinkExt, Stream, StreamExt, TryStreamExt,
};

use tokio_util::{
    codec::{Decoder, Encoder, FramedRead, FramedWrite},
    compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt},
};

pub struct Body<'a> {
    stream: BoxStream<'a, Result<Frame, std::io::Error>>,
}

impl<'a> Body<'a> {
    pub fn new(
        stream: impl Stream<Item = Result<Frame, std::io::Error>> + Send + Unpin + 'a,
    ) -> Self {
        Self {
            stream: stream.boxed(),
        }
    }

    pub fn new_empty() -> Self {
        Self::new(tokio_stream::once(Ok(Frame::End)).boxed())
    }

    pub fn from_bytes_stream(stream: impl Stream<Item = Bytes> + Send + Unpin + 'a) -> Self {
        let stream = stream.map(Ok);
        Self::from_result_stream(stream)
    }

    pub fn from_result_stream(
        stream: impl Stream<Item = Result<Bytes, std::io::Error>> + Send + Unpin + 'a,
    ) -> Self {
        let stream = stream
            .map_ok(Frame::Data)
            .chain(stream::once(async { Ok(Frame::End) }))
            .boxed();
        Self { stream }
    }

    pub async fn bytes(&mut self) -> Result<Bytes, std::io::Error> {
        let mut bytes = BytesMut::new();
        while let Some(item) = self.bytes_stream().next().await {
            bytes.extend(item?);
        }
        Ok(bytes.freeze())
    }

    pub fn bytes_stream(&mut self) -> BoxStream<'_, Result<Bytes, std::io::Error>> {
        let stream = stream! {
            while let Some(item) = self.stream.next().await {
                match item? {
                    Frame::End => return,
                    Frame::Data(data) => {
                        yield Ok(data);
                    }
                }
            }
        };
        Box::pin(stream)
    }
}

impl<'a> From<BoxStream<'a, Result<Bytes, std::io::Error>>> for Body<'a> {
    fn from(stream: BoxStream<'a, Result<Bytes, std::io::Error>>) -> Self {
        Self::from_result_stream(stream)
    }
}

impl<'a> From<Bytes> for Body<'a> {
    fn from(bytes: Bytes) -> Self {
        Self::from_bytes_stream(tokio_stream::once(bytes))
    }
}

// impl<'a> Transport for Body<'a> {
//     type Error = std::io::Error;
//     async fn from_reader<'b>(
//         io: impl tokio::io::AsyncRead + Send + Sync + Unpin + 'b + 'a,
//     ) -> Result<Option<Self>, Self::Error>
//     where
//         Self: Sized + 'b,
//         'b: 'a,
//     {
//         let stream = FramedRead::new(io, FrameCodec);
//         let body = Self::new(stream);
//         Ok(Some(body))
//     }

//     async fn write_into(&mut self, io: impl AsyncWrite + Send + Unpin) -> Result<(), Self::Error> {
//         let mut framed = FramedWrite::new(io, FrameCodec);
//         while let Some(frame) = self.stream.next().await {
//             framed.send(frame?).await?;
//         }
//         framed.send(Frame::End).await?;
//         Ok(())
//     }
// }

#[derive(Debug, Clone)]
pub enum Frame {
    End,
    Data(Bytes),
}

#[async_trait]
impl Message<'_> for Frame {
    type Error = std::io::Error;
    async fn from_reader(
        io: impl futures::AsyncRead + Send + Unpin,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized,
    {
        let mut framed = FramedRead::new(io.compat(), FrameCodec);
        framed.next().await.transpose()
    }

    async fn write_in(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> Result<(), Self::Error> {
        let mut framed = FramedWrite::new(io.compat_write(), FrameCodec);
        framed.send(self.clone()).await?;
        Ok(())
    }
}

struct FrameCodec;

impl FrameCodec {
    pub const LENTH_SIZE: usize = 8;
    pub const TAG_SIZE: usize = 1;
    pub const END_TAG: u8 = 0;
    pub const DATA_TAG: u8 = 1;
}

impl Decoder for FrameCodec {
    type Item = Frame;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < FrameCodec::LENTH_SIZE {
            return Ok(None);
        }

        let tag = src.get_u8();
        if tag == 0 {
            return Ok(Some(Frame::End));
        } else if tag != 1 {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }

        let len = src.get_u64_le() as usize;

        if src.len() < len {
            return Ok(None);
        }

        let bytes = src.split_to(len);

        Ok(Some(Frame::Data(bytes.freeze())))
    }
}

impl Encoder<Frame> for FrameCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Frame, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        self.encode(&item, dst)
    }
}

impl Encoder<&Frame> for FrameCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: &Frame, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.reserve(FrameCodec::TAG_SIZE);
        match item {
            Frame::End => dst.put_u8(FrameCodec::END_TAG),
            Frame::Data(data) => {
                dst.put_u8(FrameCodec::DATA_TAG);
                dst.reserve(data.len() + FrameCodec::LENTH_SIZE);
                dst.put_u64_le(data.len() as u64);
                dst.extend_from_slice(&data);
            }
        }
        Ok(())
    }
}

impl Encoder<&mut Frame> for FrameCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: &mut Frame, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        self.encode(&*item, dst)
    }
}
