#![allow(unused)]

use futures_lite::{AsyncWrite, AsyncWriteExt};
use zerocopy::IntoBytes;

use crate::{
    body::Body,
    code::Code,
    error::{Error, Result},
    message::FromBody,
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

    pub(crate) async fn write_into(self, writer: &mut (impl AsyncWrite + Unpin)) -> Result<()> {
        writer.write_all(self.code.as_bytes()).await?;
        todo!()
    }
}
