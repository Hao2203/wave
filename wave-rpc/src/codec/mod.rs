use std::{future::Future, io};
use tokio::io::{AsyncRead, AsyncWrite};

pub trait CodecRead<T> {
    fn codec_read(
        &self,
        reader: &mut (impl AsyncRead + Unpin + ?Sized),
    ) -> impl Future<Output = Result<T, io::Error>> + Send;
}

pub trait CodecWrite<T> {
    fn codec_write(
        &self,
        writer: &mut (impl AsyncWrite + Unpin + ?Sized),
        value: T,
    ) -> impl Future<Output = Result<(), io::Error>> + Send;
}
