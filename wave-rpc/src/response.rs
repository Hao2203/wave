#![allow(unused)]
use async_trait::async_trait;
use futures::AsyncRead;

use crate::{
    body::Body,
    code::Code,
    error::{Error, Result},
    message::{FromReader, SendTo},
};

pub struct Response<T> {
    code: Code,
    body: T,
}

impl<T> Response<T> {
    pub fn new(code: Code, body: T) -> Self {
        Self { code, body }
    }

    pub fn success(body: T) -> Self {
        Self::new(Code::Ok, body)
    }

    pub fn code(&self) -> Code {
        self.code
    }

    pub fn is_success(&self) -> bool {
        self.code == Code::Ok
    }
}

impl Response<Body<'_>> {
    pub fn from_error(err: Error) -> Self {
        let code = err.code();
        let body = Body::new(err);
        Self::new(code, body)
    }
}

#[async_trait]
impl<'a, T> FromReader<'a> for Response<T>
where
    T: FromReader<'a>,
{
    type Error = std::io::Error;

    async fn from_reader(
        mut reader: impl AsyncRead + Send + Unpin + 'a,
    ) -> Result<Self, Self::Error> {
        let code = Code::from_reader(&mut reader).await?;
        let body = T::from_reader(reader).await.unwrap();
        Ok(Response { code, body })
    }
}

#[async_trait]
impl<T> SendTo for Response<T>
where
    T: SendTo<Error: Into<Error>> + Send,
{
    type Error = std::io::Error;

    async fn send_to(
        &mut self,
        io: &mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> std::result::Result<(), Self::Error> {
        self.code.send_to(io).await?;
        let res = self.body.send_to(io).await.map_err(Into::into);
        if let Err(mut e) = res {
            e.send_to(io).await?;
        }
        Ok(())
    }
}
