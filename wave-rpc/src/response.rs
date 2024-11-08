use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    body_stream::Body,
    error::{Code, Error, Result},
    transport::Transport,
};

pub struct Response<'a> {
    code: u16,
    body: Body<'a>,
}

impl<'a> Response<'a> {
    pub const CODE_SIZE: usize = 2;
    pub const SUCCESS_CODE: u16 = 0;

    pub fn new(code: u16, body: Body<'a>) -> Self {
        Self { body, code }
    }

    pub fn success(body: Body<'a>) -> Self {
        Self::new(0, body)
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn is_success(&self) -> bool {
        self.code == Self::SUCCESS_CODE
    }

    pub fn error_code(&self) -> Result<Code, Error> {
        Code::try_from(self.code())
    }

    pub fn code(&self) -> u16 {
        self.code
    }

    pub fn into_body(self) -> Body<'a> {
        self.body
    }

    pub fn body_mut(&mut self) -> &mut Body<'a> {
        &mut self.body
    }
}

impl<'a> Transport<'a> for Response<'a> {
    type Error = Error;

    async fn from_reader(
        mut reader: impl AsyncRead + Send + Sync + Unpin + 'a,
    ) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized,
    {
        let code = reader.read_u16_le().await?;
        let body = Body::from_reader(reader).await?;
        let resp = body.map(|body| Response::new(code, body));
        Ok(resp)
    }

    async fn write_into(
        &mut self,
        mut io: impl AsyncWrite + Send + Sync + Unpin,
    ) -> Result<(), Self::Error> {
        io.write_u16_le(self.code).await?;
        self.body.write_into(io).await?;
        Ok(())
    }
}
