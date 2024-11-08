#![allow(unused)]
use crate::body_stream::Body;
use bytes::{Buf, Bytes, BytesMut};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    future::Future,
    marker::PhantomData,
};

pub trait Message: Sized {
    type Error: Debug + Display;

    fn into_body<'a>(self) -> Result<Body<'a>, Self::Error>;

    fn from_body(body: &mut Body<'_>) -> impl Future<Output = Result<Self, Self::Error>> + Send;
}

pub struct Bincode<T>(pub T);

impl<T> Message for Bincode<T>
where
    T: Serialize + for<'de> Deserialize<'de> + std::marker::Send,
{
    type Error = bincode::Error;

    async fn from_body(body: &mut Body<'_>) -> Result<Self, Self::Error> {
        let bytes = body.bytes().await?;
        bincode::deserialize(bytes.as_ref()).map(Self)
    }

    fn into_body<'a>(self) -> Result<Body<'a>, Self::Error> {
        let bytes = bincode::serialize(&self.0)?;
        Ok(Body::from(Bytes::from(bytes)))
    }
}

pub mod stream {
    #![allow(unused)]
    use crate::body_stream::Body;
    use bytes::{Buf, Bytes, BytesMut};
    use futures::StreamExt;
    use serde::{Deserialize, Serialize};
    use std::{
        fmt::{Debug, Display},
        future::Future,
        marker::PhantomData,
    };

    pub trait Message: Sized {
        type Error: Debug + Display;

        fn into_body<'a>(self) -> Result<Body<'a>, Self::Error>;

        fn from_body(body: &mut Body<'_>)
            -> impl Future<Output = Result<Self, Self::Error>> + Send;
    }

    pub struct Bincode<T>(pub T);

    impl<T> Message for Bincode<T>
    where
        T: Serialize + for<'de> Deserialize<'de> + std::marker::Send,
    {
        type Error = bincode::Error;

        async fn from_body(body: &mut Body<'_>) -> Result<Self, Self::Error> {
            let bytes = body.bytes().await?;
            bincode::deserialize(bytes.as_ref()).map(Self)
        }

        fn into_body<'a>(self) -> Result<Body<'a>, Self::Error> {
            let bytes = bincode::serialize(&self.0)?;
            Ok(Body::from(Bytes::from(bytes)))
        }
    }
}
