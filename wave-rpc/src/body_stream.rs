#![allow(unused)]
use crate::error::Error;
use async_stream::stream;
use async_trait::async_trait;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{
    stream::{self, BoxStream},
    AsyncRead, AsyncReadExt, AsyncWrite, SinkExt, Stream, StreamExt, TryStreamExt,
};

use tokio_util::{
    codec::{Decoder, Encoder, FramedRead, FramedWrite},
    compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt},
};

// pub struct Body<'a> {
//     message: Box<dyn Message<'a, Error = Error> + 'a>,
// }

// impl<'a> Body<'a> {
//     pub fn new(message: impl Message<'a, Error: Into<Error>> + Send + 'a) -> Self {
//         Self {
//             message: message.into_boxed(),
//         }
//     }
// }
