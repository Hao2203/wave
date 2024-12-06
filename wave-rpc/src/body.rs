#![allow(unused)]
use crate::error::{BoxError, Error, Result};
use async_trait::async_trait;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures_lite::{
    stream::{self, Boxed},
    AsyncWrite, Stream, StreamExt as _,
};
use tokio_util::codec::{Decoder, Encoder};

pub trait MessageBody: Stream<Item = Result<Bytes, Self::Error>> + Send + 'static {
    type Error: Into<BoxError>;
}

impl<T, E> MessageBody for T
where
    T: Stream<Item = Result<Bytes, E>> + Send + 'static,
    E: Into<BoxError>,
{
    type Error = E;
}

pub struct Body {
    is_end_of_stream: bool,
    framed_stream: Boxed<Result<Frame, BoxError>>,
}

impl Body {
    pub fn new(message_body: impl MessageBody) -> Self {
        let framed_stream = message_body
            .filter(|data| {
                if let Ok(data) = data {
                    !data.is_empty() // if empty, remove it
                } else {
                    true // if error, we don't care
                }
            })
            .map(|data| data.map(Frame::new).map_err(Into::into))
            .chain(stream::once(Ok(Frame::new_empty()))) // end of stream
            .boxed();
        Self {
            is_end_of_stream: false,
            framed_stream,
        }
    }

    pub fn once(data: Bytes) -> Self {
        Self::new(stream::once(Ok::<_, Error>(data)))
    }

    pub fn is_end_of_stream(&self) -> bool {
        self.is_end_of_stream
    }

    pub(crate) fn framed_stream(self) -> Boxed<Result<Frame, BoxError>> {
        self.framed_stream
    }
}

impl futures_lite::Stream for Body {
    type Item = Result<Bytes, BoxError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        // if self.is_end_of_stream {
        //     return std::task::Poll::Ready(None);
        // }
        // let frame = self.framed_stream.poll_next(cx);
        // frame.map(|f| {
        //     f.map(|frame| {
        //         frame.map(|frame| {
        //             self.is_end_of_stream = frame.is_end_of_stream();
        //             frame.data
        //         })
        //     })
        // })
        todo!()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Frame {
    End,
    Data(Bytes),
}

impl Frame {
    const SIZE_LEN: usize = 4;
    const EOS_LEN: usize = 1;

    pub fn new(data: Bytes) -> Frame {
        Frame::Data(data)
    }

    pub fn new_empty() -> Frame {
        Self::new(Bytes::new())
    }

    pub fn is_end_of_stream(&self) -> bool {
        matches!(self, Frame::End)
    }
}

pub(crate) struct FrameCodec;

impl Encoder<Frame> for FrameCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Frame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(Frame::EOS_LEN);
        match item {
            Frame::End => dst.put_u8(0),
            Frame::Data(data) => {
                let data_size = data.len() as u32;
                dst.put_u8(1);
                dst.reserve(Frame::SIZE_LEN + data_size as usize);
                dst.put_u32(data_size);
                dst.put(data);
            }
        };
        Ok(())
    }
}

impl Decoder for FrameCodec {
    type Error = std::io::Error;
    type Item = Frame;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Frame>, Self::Error> {
        if src.len() < Frame::EOS_LEN {
            return Ok(None);
        }

        let is_end_of_stream = src.get_u8() != 0;
        if is_end_of_stream {
            return Ok(Some(Frame::End));
        }

        if src.len() < Frame::SIZE_LEN {
            return Ok(None);
        }
        let data_size = src.get_u32();

        if src.len() < data_size as usize {
            return Ok(None);
        }

        let frame = Frame::new(src.split_to(data_size as usize).freeze());

        Ok(Some(frame))
    }
}
