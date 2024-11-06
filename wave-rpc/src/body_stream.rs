#![allow(unused)]

use async_stream::stream;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{
    stream::{self, BoxStream},
    Stream, StreamExt, TryStreamExt,
};
use std::ops::Deref;
use tokio_util::codec::{Decoder, Encoder, Framed, FramedRead};

use crate::Transport;

pub struct Body<'a> {
    stream: BoxStream<'a, Result<Frame, std::io::Error>>,
}

impl<'a> Body<'a> {
    pub fn new(
        mut stream: impl Stream<Item = Result<Bytes, std::io::Error>> + Send + Unpin + 'a,
    ) -> Self {
        Self::from(Box::pin(stream) as BoxStream<_>)
    }

    pub fn from_bytes_stream(mut stream: impl Stream<Item = Bytes> + Send + Unpin + 'a) -> Self {
        let stream = stream.map(Ok);
        Self::from(Box::pin(stream) as BoxStream<_>)
    }

    pub async fn bytes(&mut self) -> Result<Bytes, std::io::Error> {
        let mut bytes = BytesMut::new();
        while let Some(item) = self.stream.next().await {
            bytes.extend(item?.0);
        }
        Ok(bytes.freeze())
    }

    fn from_bytes(bytes: Bytes) -> Self {
        Self {
            stream: stream::once(async { Ok(Frame(bytes)) }).boxed(),
        }
    }
}

impl<'a> From<BoxStream<'a, Result<Bytes, std::io::Error>>> for Body<'a> {
    fn from(stream: BoxStream<'a, Result<Bytes, std::io::Error>>) -> Self {
        let stream = stream
            .map(|bytes| bytes.map(Frame::from))
            .chain(stream::once(async { Ok(Frame(Bytes::new())) }))
            .boxed();
        Self { stream }
    }
}

impl From<Bytes> for Body<'_> {
    fn from(bytes: Bytes) -> Self {
        Self::from_bytes(bytes)
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

pub struct BodyTransport {}

impl<'a> Transport<'a> for BodyTransport {
    type Item = Body<'a>;
    fn stream(
        &mut self,
        io: impl tokio::io::AsyncRead + Send + Sync + Unpin + 'a,
    ) -> impl Stream<Item = crate::Result<Self::Item>> + Unpin + Send + 'a {
        stream! {
            let codec = FrameCodec;
            let framed = FramedRead::new(io, codec);
            let body = Body {
                stream: framed.into_stream().boxed()
            };
            yield body
        }
    }

    fn sink(
        &mut self,
        io: impl tokio::io::AsyncWrite + Send + Sync + Unpin + 'a,
    ) -> impl futures::Sink<crate::Result<Self::Item>> + Unpin + Send + 'a {
        todo!()
    }
}

pub struct Frame(pub Bytes);

impl From<Bytes> for Frame {
    fn from(bytes: Bytes) -> Self {
        Self(bytes)
    }
}

impl From<Frame> for Bytes {
    fn from(frame: Frame) -> Self {
        frame.0
    }
}

impl Deref for Frame {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
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

        Ok(Some(bytes.freeze().into()))
    }
}

impl Encoder<Bytes> for FrameCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Bytes, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.reserve(item.len() + FrameCodec::LENTH_SIZE);
        dst.put_u64_le(item.len() as u64);
        dst.extend_from_slice(&item);
        Ok(())
    }
}
