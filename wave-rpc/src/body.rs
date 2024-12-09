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
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite};

pub type BoxMessageBody =
    Pin<Box<dyn MessageBody<Error = BoxError, Item = Result<Arc<[u8]>, BoxError>>>>;

pub trait MessageBody:
    Stream<Item = Result<Arc<[u8]>, Self::Error>> + Unpin + Send + 'static
{
    type Error: Into<BoxError> + Send + Sync;
}

impl<T, E> MessageBody for T
where
    T: Stream<Item = Result<Arc<[u8]>, E>> + Send + 'static + Unpin,
    E: Into<BoxError> + Send + Sync,
{
    type Error = E;
}

pub struct Body {
    frame_stream: Boxed<Result<Frame, BoxError>>,
}

impl Body {
    pub fn new(message_body: impl MessageBody) -> Self {
        let frame_stream = message_body
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
            .map(|data| data.map(Frame::new).map_err(Into::into))
            .chain(stream::once(Ok(Frame::new_empty()))) // end of stream
            .boxed();
        Self { frame_stream }
    }

    pub fn once(data: Arc<[u8]>) -> Self {
        Self::new(stream::once(Ok::<_, Error>(data)))
    }

    pub(crate) fn from_reader(reader: impl AsyncRead + Unpin + Send + 'static) -> Self {
        let frame_stream = FramedRead::new(reader.compat(), FrameCodec)
            .map(|frame| frame.map_err(Into::into))
            .boxed();
        Self { frame_stream }
    }

    pub(crate) async fn write_into(
        mut self,
        writer: &mut (impl AsyncWrite + Unpin),
    ) -> Result<(), BoxError> {
        let mut encoder = Frame::codec();
        while let Some(frame) = self.frame_stream.next().await {
            let frame = frame?;
            let mut buf = BytesMut::new();
            encoder.encode(frame, &mut buf);
            writer.write_all(&buf).await?;
        }
        Ok(())
    }

    pub(crate) fn into_message_body(
        self,
    ) -> impl MessageBody<Error = BoxError, Item = Result<Arc<[u8]>, BoxError>> {
        let mut is_end_of_stream = false;
        self.frame_stream.filter_map(move |frame| {
            if is_end_of_stream {
                None
            } else {
                match frame {
                    Ok(Frame::Data(data)) => Some(Ok(Vec::from(data).into())),
                    Ok(Frame::End) => {
                        is_end_of_stream = true;
                        None
                    }
                    Err(err) => Some(Err(err)),
                }
            }
        })
    }

    pub(crate) fn framed_stream(self) -> Boxed<Result<Frame, BoxError>> {
        self.frame_stream
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
pub(crate) enum Frame {
    End,
    Data(Bytes),
}

impl Frame {
    const SIZE_LEN: usize = 4;
    const EOS_LEN: usize = 1;

    pub(crate) fn codec() -> FrameCodec {
        FrameCodec
    }

    pub fn new(data: Arc<[u8]>) -> Frame {
        Frame::Data(Bytes::from_owner(data))
    }

    pub(crate) async fn from_connection_reader(
        reader: &mut ConnectionManager,
    ) -> crate::error::Result<Frame> {
        let eos = reader.get_u8().await?;
        match eos {
            0 => Ok(Frame::End),
            1 => {
                let data_size = reader.get_u32().await?;
                let data = reader.read(data_size as usize).await?;
                Ok(Frame::Data(data.into()))
            }
            _ => Err(io::Error::from(io::ErrorKind::InvalidData).into()),
        }
    }

    pub fn new_empty() -> Frame {
        Self::new(Arc::new([]))
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
