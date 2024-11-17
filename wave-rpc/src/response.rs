#![allow(unused)]
use async_trait::async_trait;
use futures::AsyncRead;

use crate::{
    code::Code,
    error::{Error, Result},
    message::{FromReader, WriteIn},
};

pub struct Response<T> {
    code: Code,
    body: T,
}

impl<T> Response<T> {
    pub fn new(code: Code, body: T) -> Self {
        Self { code, body }
    }

    pub fn is_success(&self) -> bool {
        self.code == Code::Ok
    }
}

#[async_trait]
impl<'a, T> FromReader<'a> for Response<T>
where
    T: FromReader<'a>,
{
    type Error = Error;

    async fn from_reader(
        mut reader: impl AsyncRead + Send + Unpin + 'a,
    ) -> Result<Self, Self::Error> {
        let code = Code::from_reader(&mut reader).await?;
        let body = T::from_reader(reader).await.unwrap();
        Ok(Response { code, body })
    }
}

#[async_trait]
impl<T> WriteIn for Response<T>
where
    T: WriteIn + Send,
{
    type Error = Error;

    async fn write_in(
        &mut self,
        io: &mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> std::result::Result<(), Self::Error> {
        self.code.write_in(io).await?;
        self.body.write_in(io).await.unwrap();
        Ok(())
    }
}
