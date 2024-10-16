use std::task::Poll;

use anyhow::Result;
use bytes::Bytes;
use futures::{stream::BoxStream, Stream};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

#[non_exhaustive]
pub enum Body<'conn> {
    Bytes(Bytes),
    Stream(BoxStream<'conn, Result<Bytes>>),
}

#[derive(Clone, Copy, IntoBytes, TryFromBytes, KnownLayout, Immutable, Unaligned)]
#[repr(u8)]
#[non_exhaustive]
pub enum BodyType {
    Bytes,
    Stream,
}

impl Stream for Body<'_> {
    type Item = Result<Bytes>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.get_mut() {
            Body::Bytes(bytes) => {
                if bytes.is_empty() {
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(Ok(bytes.split_to(bytes.len()))))
                }
            }
            Body::Stream(stream) => stream.as_mut().poll_next(cx),
        }
    }
}
