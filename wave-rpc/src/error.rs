use crate::{code::Code, message::FromReader};
use derive_more::derive::Display;
use std::any::Any;
use std::fmt::{Debug, Display};

pub type Result<T, E = Error> = core::result::Result<T, E>;
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, Display, derive_more::Error)]
pub struct Error {
    cause: Box<dyn RpcError + Send + Sync>,
}

impl Error {
    pub fn as_rpc_error(&self) -> &dyn RpcError {
        self.cause.as_ref()
    }

    pub fn as_error<T: RpcError + 'static>(&self) -> Option<&T> {
        <dyn Send + Any>::downcast_ref(&self.cause)
    }
}

impl FromReader<'_> for Error {
    type Error = Self;

    async fn from_reader(
        mut reader: impl futures::AsyncRead + Send + Unpin,
    ) -> std::result::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let code = Code::from_reader(&mut reader).await?;
        let message = String::from_reader(&mut reader).await?;
        Ok(ErrorMsg::new(code, message).into())
    }
}

pub trait RpcError: Display + Debug {
    fn code(&self) -> Code;

    fn message(&self) -> String {
        self.to_string()
    }
}

impl<T: RpcError + Send + Sync + 'static> From<T> for Error {
    fn from(err: T) -> Self {
        Self {
            cause: Box::new(err),
        }
    }
}

impl RpcError for std::io::Error {
    fn code(&self) -> Code {
        Code::IoError
    }
}

#[derive(Debug, Display)]
#[display("code: {}; message: {}", self.code, self.message)]
pub struct ErrorMsg {
    pub code: Code,
    pub message: String,
}

impl ErrorMsg {
    pub fn new(code: Code, message: String) -> Self {
        Self { code, message }
    }
}

impl RpcError for ErrorMsg {
    fn code(&self) -> Code {
        self.code
    }
}
