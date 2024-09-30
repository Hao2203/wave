use std::{future::Future, io, pin::Pin};
use tokio::io::AsyncRead;

pub trait FromAsyncRead: Sized {
    fn from_async_read(
        reader: Pin<&mut (impl AsyncRead + ?Sized)>,
    ) -> impl Future<Output = Result<Self, io::Error>> + Send;
}

pub trait WriteResponse: Sized {
    fn async_write(
        self,
        writer: Pin<&mut (impl AsyncRead + ?Sized)>,
    ) -> impl Future<Output = Result<Self, io::Error>> + Send;
}

pub type RefReader<'a> = Pin<&'a mut (dyn AsyncRead + Send + 'a)>;

pub type RefWriter<'a> = Pin<&'a mut (dyn AsyncRead + Send + 'a)>;

pub type BoxReader<'a> = Pin<Box<dyn AsyncRead + Send + 'a>>;

pub type BoxWriter<'a> = Pin<Box<dyn AsyncRead + Send + 'a>>;
