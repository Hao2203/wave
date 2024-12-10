// #![allow(unused)]
use crate::code::Code;
use derive_more::derive::Display;
use std::{
    any::Any,
    convert::Infallible,
    fmt::{Debug, Display},
    io,
    sync::Arc,
};
use zerocopy::TryFromBytes;

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

    pub fn code(&self) -> Code {
        self.as_rpc_error().code()
    }
}

pub trait RpcError: Display + Debug + Send + Sync + 'static {
    fn code(&self) -> Code;

    fn to_bytes(&self) -> Arc<[u8]> {
        Arc::from(self.to_string().as_bytes())
    }
}

impl<T: RpcError + 'static> From<T> for Error {
    fn from(value: T) -> Self {
        Error {
            cause: Box::new(value),
        }
    }
}

impl<Src, Dst> From<zerocopy::TryReadError<Src, Dst>> for Error
where
    Dst: ?Sized + TryFromBytes,
{
    fn from(err: zerocopy::TryReadError<Src, Dst>) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, format!("{:?}", err)).into()
    }
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        panic!("{:?}", err)
    }
}

impl RpcError for io::Error {
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

    fn to_bytes(&self) -> Arc<[u8]> {
        Arc::from(self.to_string().as_bytes())
    }
}
