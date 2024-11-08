#![allow(unused)]

use async_stream::stream;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{
    sink,
    stream::{self, BoxStream},
    SinkExt, Stream, StreamExt, TryStream, TryStreamExt,
};
use std::ops::Deref;
use tokio_util::codec::{Decoder, Encoder, Framed, FramedRead, FramedWrite};

use crate::{error::Error, transport::Transport};

pub struct Body<'a> {
    stream: BoxStream<'a, Result<Frame, std::io::Error>>,
}

impl<'a> Body<'a> {
    pub fn new(
        mut stream: impl Stream<Item = Result<Frame, std::io::Error>> + Send + Unpin + 'a,
    ) -> Self {
        Self {
            stream: stream.boxed(),
        }
    }

    pub fn new_empty() -> Self {
        Self::new(stream::once(async { Ok(Frame(Bytes::new())) }).boxed())
    }

    pub fn from_bytes_stream(mut stream: impl Stream<Item = Bytes> + Send + Unpin + 'a) -> Self {
        let stream = stream.map(Ok);
        Self::from_result_stream(stream)
    }

    pub fn from_result_stream(
        mut stream: impl Stream<Item = Result<Bytes, std::io::Error>> + Send + Unpin + 'a,
    ) -> Self {
        let stream = stream.map_ok(Frame).boxed();
        Self { stream }
    }

    pub async fn bytes(&mut self) -> Result<Bytes, std::io::Error> {
        let mut bytes = BytesMut::new();
        while let Some(item) = self.stream.next().await {
            bytes.extend(item?.0);
        }
        Ok(bytes.freeze())
    }
}

impl<'a> From<BoxStream<'a, Result<Bytes, std::io::Error>>> for Body<'a> {
    fn from(stream: BoxStream<'a, Result<Bytes, std::io::Error>>) -> Self {
        Self::from_result_stream(stream)
    }
}

impl Stream for Body<'_> {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx).map_ok(|frame| frame.0)
    }
}

impl<'a> Transport<'a> for Body<'a> {
    type Error = std::io::Error;
    async fn from_reader(
        io: impl tokio::io::AsyncRead + Send + Sync + Unpin + 'a,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized,
    {
        let stream = FramedRead::new(io, FrameCodec);
        let body = Self::new(stream);
        Ok(Some(body))
    }

    async fn write_into(
        &mut self,
        io: impl tokio::io::AsyncWrite + Send + Sync + Unpin,
    ) -> Result<(), Self::Error> {
        let mut framed = FramedWrite::new(io, FrameCodec);
        while let Some(frame) = self.stream.next().await {
            framed.send(frame?.into()).await?;
        }
        framed.send(Frame(Bytes::new())).await?;
        framed.close().await?;
        Ok(())
    }
}
pub struct Frame(pub Bytes);

impl From<Frame> for Bytes {
    fn from(frame: Frame) -> Self {
        frame.0
    }
}

impl From<Bytes> for Frame {
    fn from(bytes: Bytes) -> Self {
        Self(bytes)
    }
}

impl Deref for Frame {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Transport<'_> for Frame {
    type Error = std::io::Error;
    async fn from_reader(
        io: impl tokio::io::AsyncRead + Send + Sync + Unpin,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized,
    {
        let mut framed = FramedRead::new(io, FrameCodec);
        framed.next().await.transpose()
    }

    async fn write_into(
        &mut self,
        io: impl tokio::io::AsyncWrite + Send + Sync + Unpin,
    ) -> Result<(), Self::Error> {
        let mut framed = FramedWrite::new(io, FrameCodec);
        framed.send(self.0.split_to(self.0.len()).into()).await?;
        framed.flush().await?;
        Ok(())
    }
}

struct FrameCodec;

impl FrameCodec {
    pub const LENTH_SIZE: usize = 8;
}

impl Decoder for FrameCodec {
    type Item = Frame;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < FrameCodec::LENTH_SIZE {
            return Ok(None);
        }

        let len = src.get_u64_le() as usize;

        if src.len() < len {
            return Ok(None);
        }

        let bytes = src.split_to(len);

        Ok(Some(Frame(bytes.freeze())))
    }
}

impl Encoder<Frame> for FrameCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Frame, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.reserve(item.len() + FrameCodec::LENTH_SIZE);
        dst.put_u64_le(item.len() as u64);
        dst.extend_from_slice(&item);
        Ok(())
    }
}
