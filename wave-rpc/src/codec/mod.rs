use std::{future::Future, io, pin::Pin};
use tokio::io::{AsyncRead, AsyncWrite};

pub trait CodecRead<T> {
    fn codec_read(
        &self,
        reader: Pin<&mut (impl AsyncRead + ?Sized)>,
    ) -> impl Future<Output = Result<T, io::Error>> + Send;
}

pub trait CodecWrite<T> {
    fn codec_write(
        &self,
        writer: Pin<&mut (impl AsyncWrite + ?Sized)>,
        value: T,
    ) -> impl Future<Output = Result<(), io::Error>> + Send;
}
