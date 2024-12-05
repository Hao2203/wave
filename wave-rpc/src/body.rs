#![allow(unused)]
use crate::error::Result;
use async_trait::async_trait;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures_lite::{
    stream::{self, Boxed},
    AsyncWrite, Stream, StreamExt as _,
};
use tokio_util::codec::{Decoder, Encoder};

pub struct Body {
    is_end_of_stream: bool,
    framed_stream: Boxed<Result<Frame>>,
}

impl Body {
    pub fn from_stream(stream: impl Stream<Item = Result<Bytes>> + Send + 'static) -> Self {
        let framed_stream = stream.map(|data| data.map(Frame::new)).boxed();
        Self {
            is_end_of_stream: false,
            framed_stream,
        }
    }

    pub fn once(data: Bytes) -> Self {
        Self {
            is_end_of_stream: false,
            framed_stream: stream::once(Ok(Frame::new(data))).boxed(),
        }
    }

    pub fn is_end_of_stream(&self) -> bool {
        self.is_end_of_stream
    }

    pub fn framed_stream(self) -> Boxed<Result<Frame>> {
        self.framed_stream
    }
}

impl futures_lite::Stream for Body {
    type Item = Result<Bytes>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.is_end_of_stream {
            return std::task::Poll::Ready(None);
        }
        let frame = self.framed_stream.poll_next(cx);
        frame.map(|f| {
            f.map(|frame| {
                frame.map(|frame| {
                    self.is_end_of_stream = frame.is_end_of_stream();
                    frame.data
                })
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    // if data_size == 0, it means end of stream
    data_size: u32,
    data: Bytes,
}

impl Frame {
    const SIZE_LEN: usize = 4;

    pub fn new(data: Bytes) -> Frame {
        Frame {
            data_size: data.len() as u32,
            data,
        }
    }

    pub fn new_empty() -> Frame {
        Frame {
            data_size: 0,
            data: Bytes::new(),
        }
    }

    pub fn is_end_of_stream(&self) -> bool {
        self.data_size == 0
    }
}

pub struct FrameCodec;

impl Encoder<Frame> for FrameCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Frame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(Frame::SIZE_LEN);
        dst.put_u32(item.data_size);
        dst.reserve(item.data.len());
        dst.put(item.data);
        Ok(())
    }
}

impl Decoder for FrameCodec {
    type Error = std::io::Error;
    type Item = Frame;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Frame>, Self::Error> {
        if src.len() < Frame::SIZE_LEN {
            return Ok(None);
        }

        let data_size = src.get_u32();

        if src.len() < data_size as usize {
            return Ok(None);
        }

        let frame = if data_size == 0 {
            Frame::new_empty()
        } else {
            Frame::new(src.split_to(data_size as usize).freeze())
        };

        Ok(Some(frame))
    }
}
