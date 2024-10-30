#![allow(unused)]

use async_stream::stream;
use bytes::{Buf, BufMut, Bytes};
use futures::{stream::BoxStream, Stream, StreamExt};
use tokio_util::codec::{Decoder, Encoder};

pub struct Body<'a> {
    stream: BoxStream<'a, Result<Bytes, std::io::Error>>,
}

impl<'a> Body<'a> {
    pub fn new(
        mut stream: impl Stream<Item = Result<Bytes, std::io::Error>> + Send + Unpin + 'a,
    ) -> Self {
        let stream = stream! {
            while let Some(item) = stream.next().await {
                if let Ok(item) = &item {
                    if item.is_empty() {
                        break;
                    }
                }
                yield item
            }
        };
        Self::from(Box::pin(stream) as BoxStream<_>)
    }

    pub fn from_bytes_stream(mut stream: impl Stream<Item = Bytes> + Send + Unpin + 'a) -> Self {
        let stream = stream.map(Ok);
        Self::from(Box::pin(stream) as BoxStream<_>)
    }
}

impl<'a> From<BoxStream<'a, Result<Bytes, std::io::Error>>> for Body<'a> {
    fn from(stream: BoxStream<'a, Result<Bytes, std::io::Error>>) -> Self {
        Self { stream }
    }
}

impl Stream for Body<'_> {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx)
    }
}

struct ChunkCodec;

impl ChunkCodec {
    pub const LENTH_SIZE: usize = 8;
}

impl Decoder for ChunkCodec {
    type Item = Bytes;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < ChunkCodec::LENTH_SIZE {
            return Ok(None);
        }

        let len = src.get_u64_le() as usize;
        if src.len() < ChunkCodec::LENTH_SIZE {
            return Ok(None);
        }

        let bytes = src.split_to(len);

        Ok(Some(bytes.freeze()))
    }
}

impl Encoder<Bytes> for ChunkCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Bytes, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.reserve(item.len() + ChunkCodec::LENTH_SIZE);
        dst.put_u64_le(item.len() as u64);
        dst.extend_from_slice(&item);
        Ok(())
    }
}
