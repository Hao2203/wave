#![allow(unused)]
use futures_lite::{AsyncRead, AsyncWrite, AsyncWriteExt};
use zerocopy::IntoBytes;

use crate::{
    body::Body,
    code::Code,
    error::{BoxError, Error, Result},
    message::FromStream,
};

pub struct Response {
    code: Code,
    body: Body,
}

impl Response {
    pub fn new(code: Code, body: Body) -> Self {
        Self { code, body }
    }

    pub fn success(body: Body) -> Self {
        Self::new(Code::Ok, body)
    }

    pub fn code(&self) -> Code {
        self.code
    }

    pub fn is_success(&self) -> bool {
        self.code == Code::Ok
    }

    pub(crate) async fn from_reader(
        mut reader: impl AsyncRead + Unpin + Send + 'static,
    ) -> Result<Self> {
        let code = Code::from_reader(&mut reader).await?;
        let body = Body::from_reader(reader);
        Ok(Response { code, body })
    }

    pub(crate) async fn write_into(
        self,
        writer: &mut (impl AsyncWrite + Unpin),
    ) -> Result<(), BoxError> {
        let Self { code, body } = self;
        code.write_into(writer).await?;
        body.write_into(writer).await?;
        Ok(())
    }
}
