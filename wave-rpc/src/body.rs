#![allow(unused)]
use crate::{
    error::{BoxError, Error, Result},
    transport::ConnectionManager,
};
use async_compat::CompatExt;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures_lite::{
    stream::{self, Boxed},
    AsyncRead, AsyncWrite, AsyncWriteExt, Stream, StreamExt as _,
};
use std::{io, pin::Pin, sync::Arc};
use tokio_util::codec::{Decoder, Encoder, FramedRead};

pub type BoxMessageBody = Pin<Box<dyn MessageBody<Error = Error, Item = Result<Bytes, Error>>>>;

pub trait MessageBody: Stream<Item = Result<Bytes, Self::Error>> + Unpin + Send + 'static {
    type Error: Into<Error>;
}

impl<T, E> MessageBody for T
where
    T: Stream<Item = Result<Bytes, E>> + Send + 'static + Unpin,
    E: Into<Error>,
{
    type Error = E;
}

pub struct Body {
    inner: BoxMessageBody,
}

impl Body {
    pub fn new(message_stream: impl MessageBody) -> Self {
        Self {
            inner: Box::pin(message_stream.map(|data| data.map_err(Into::into))),
        }
    }

    pub fn from_frame_stream(stream: impl Stream<Item = Frame> + Unpin + Send + 'static) -> Self {
        let frame_stream = stream.map(|frame| todo!()).boxed();
        Self { inner: todo!() }
    }

    pub fn into_message_stream(self) -> impl Stream<Item = Bytes> {
        self.inner.filter_map(|data| data.ok())
    }

    pub fn into_frame_stream(self) -> impl Stream<Item = Frame> {
        self.inner
            .filter_map(|data| {
                if let Ok(data) = data {
                    if !data.is_empty() {
                        Some(Ok(data))
                    } else {
                        None
                    } // if empty, remove ith
                } else {
                    Some(data) // if error, we don't care
                }
            })
            .map(|data| {
                data.map(Frame::new)
                    .unwrap_or_else(|e| Frame::new(e.as_rpc_error().to_bytes()))
            })
            .chain(stream::once(Frame::new_empty())) // end of stream
            .boxed()
    }

    pub fn once(data: Bytes) -> Self {
        Self {
            inner: Box::pin(stream::once(Ok::<_, Error>(data))),
        }
    }

    pub(crate) fn from_reader(reader: impl AsyncRead + Unpin + Send + 'static) -> Self {
        // let frame_stream = FramedRead::new(reader.compat(), FrameCodec)
        //     .map(|frame| frame.map_err(Into::into))
        //     .boxed();
        todo!()
    }

    pub(crate) async fn write_into(
        mut self,
        writer: &mut (impl AsyncWrite + Unpin),
    ) -> Result<(), BoxError> {
        let mut encoder = Frame::codec();
        while let Some(frame) = self.inner.next().await {
            let frame = frame?;
            let mut buf = BytesMut::new();
            todo!();
            // encoder.encode(frame, &mut buf)?;
            writer.write_all(&buf).await?;
        }
        Ok(())
    }
}

impl<T> From<T> for Body
where
    T: MessageBody,
{
    fn from(message_body: T) -> Self {
        Self::new(message_body)
    }
}

#[derive(Debug, Clone)]
pub enum Frame {
    End,
    Data(Bytes),
}

impl Frame {
    const SIZE_LEN: usize = 4;
    const EOS_LEN: usize = 1;

    pub(crate) fn codec() -> FrameCodec {
        FrameCodec
    }

    pub fn new(data: Bytes) -> Frame {
        Frame::Data(data)
    }

    pub(crate) async fn from_connection_reader(
        reader: &mut ConnectionManager,
    ) -> crate::error::Result<Frame> {
        let eos = reader.get_u8().await.unwrap();
        match eos {
            0 => Ok(Frame::End),
            1 => {
                let data_size = reader.get_u32().await.unwrap();
                let data = reader.read(data_size as usize).await.unwrap();
                Ok(Frame::Data(data.into()))
            }
            _ => Err(io::Error::from(io::ErrorKind::InvalidData).into()),
        }
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

        let data = src.split_to(data_size as usize).freeze();
        let frame = Frame::Data(data);

        Ok(Some(frame))
    }
}
