use crate::Body;
use bytes::{BufMut, BytesMut};
use derive_more::derive::{Display, From};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

pub trait Message: Sized {
    type Error: Debug + Display;
    type Inner;

    fn from_inner(inner: Self::Inner) -> Self;

    fn into_inner(self) -> Self::Inner;

    fn into_body(self) -> Result<Body, Self::Error>;

    fn from_body(body: &mut Body) -> Result<Self, Self::Error>;
}

#[derive(From)]
pub struct Bincode<T>(T);

impl<T> Message for Bincode<T>
where
    T: for<'a> Deserialize<'a> + Serialize,
{
    type Error = bincode::Error;
    type Inner = T;

    #[inline]
    fn from_inner(inner: Self::Inner) -> Self {
        Self(inner)
    }

    #[inline]
    fn into_inner(self) -> Self::Inner {
        self.0
    }

    #[inline]
    fn into_body(self) -> Result<Body, Self::Error> {
        Ok(Body::new(bincode::serialize(&self.0)?.into()))
    }

    #[inline]
    fn from_body(body: &mut Body) -> Result<Self, Self::Error> {
        Ok(Self(bincode::deserialize(body.as_slice())?))
    }
}

#[derive(Debug, Display, From, derive_more::Error)]
pub enum MessageError {
    BoxError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[display("unexpected tag: {}", 0)]
    UnexpectedTag(#[error(ignore)] u8),
}

impl<T, E> Message for Result<T, E>
where
    T: Message<Error: core::error::Error + Send + Sync + 'static>,
    E: Message<Error: core::error::Error + Send + Sync + 'static>,
{
    type Error = MessageError;
    type Inner = Result<T::Inner, E::Inner>;

    #[inline]
    fn from_inner(inner: Self::Inner) -> Self {
        inner.map(T::from_inner).map_err(E::from_inner)
    }

    #[inline]
    fn into_inner(self) -> Self::Inner {
        self.map(T::into_inner).map_err(E::into_inner)
    }

    fn from_body(body: &mut Body) -> Result<Self, Self::Error> {
        let bytes = body.as_slice();

        let tag = bytes[0];
        if tag == 0 {
            Ok(Ok(T::from_body(body).map_err(|e| {
                Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>
            })?))
        } else if tag == 1 {
            Ok(Err(E::from_body(body).map_err(|e| Box::new(e) as Box<_>)?))
        } else {
            Err(MessageError::UnexpectedTag(tag))
        }
    }

    fn into_body(self) -> Result<Body, Self::Error> {
        let (tag, body) = match self {
            Ok(inner) => (0, T::into_body(inner).map_err(|e| Box::new(e) as Box<_>)?),
            Err(inner) => (0, E::into_body(inner).map_err(|e| Box::new(e) as Box<_>)?),
        };
        let mut bytes = BytesMut::with_capacity(1 + body.len());
        bytes.put_u8(tag);
        bytes.extend_from_slice(body.as_slice());
        Ok(Body::new(bytes.freeze()))
    }
}
