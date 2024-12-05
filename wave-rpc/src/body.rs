#![allow(unused)]
use crate::{error::Error, message::SendTo};
use async_trait::async_trait;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures_lite::{stream::Boxed, AsyncWrite, StreamExt as _};
use tokio_util::codec::{Decoder, Encoder};

pub struct Body {
    is_end_of_stream: bool,
    framed_stream: Boxed<Frame>,
}

impl Body {
    pub fn new(stream: Boxed<Frame>) -> Self {
        Self {
            is_end_of_stream: false,
            framed_stream: stream,
        }
    }

    pub fn is_end_of_stream(&self) -> bool {
        self.is_end_of_stream
    }

    pub fn framed_stream(self) -> Boxed<Frame> {
        self.framed_stream
    }
}

impl futures_lite::Stream for Body {
    type Item = Bytes;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.is_end_of_stream {
            return std::task::Poll::Ready(None);
        }
        let frame = self.framed_stream.poll_next(cx);
        frame.map(|f| {
            f.map(|f| {
                self.is_end_of_stream = f.end_of_stream;
                f.data
            })
        })
    }
}

pub struct Frame {
    data_size: u32,
    end_of_stream: bool,
    data: Bytes,
}

impl Frame {
    const SIZE_LEN: usize = 4;
}

pub struct FrameCodec;

impl Encoder<Frame> for FrameCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Frame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_u32(item.data_size);
        dst.put_u8(if item.end_of_stream { 1 } else { 0 });
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
        let end_of_stream = src.get_u8() == 1;

        if src.len() < data_size as usize {
            return Ok(None);
        }
        let data = src.split_to(data_size as usize).freeze();

        let frame = Frame {
            data_size,
            end_of_stream,
            data,
        };
        Ok(Some(frame))
    }
}
