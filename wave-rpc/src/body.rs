use anyhow::Result;
use bytes::Bytes;
use futures::stream::BoxStream;

#[non_exhaustive]
pub enum Body<'conn> {
    Bytes(Bytes),
    Stream(BoxStream<'conn, Result<Bytes>>),
}
