#![allow(unused)]
use crate::{code::Code, message::FromBody};
use async_channel::SendError;
use async_trait::async_trait;
use derive_more::derive::Display;
use std::any::Any;
use std::fmt::{Debug, Display};
use std::io;
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

    pub fn message(&self) -> String {
        self.as_rpc_error().message()
    }
}

// impl FromReader<'_> for Error {
//     type Error = Self;

//     async fn from_reader(
//         mut reader: impl futures::AsyncRead + Send + Unpin,
//     ) -> std::result::Result<Self, Self::Error>
//     where
//         Self: Sized,
//     {
//         let code = Code::from_reader(&mut reader).await?;
//         let message = String::from_reader(&mut reader).await?;
//         Ok(ErrorMsg::new(code, message).into())
//     }
// }

// #[async_trait]
// impl SendTo for Error {
//     type Error = std::io::Error;

//     async fn send_to(
//         &mut self,
//         io: &mut (dyn futures::AsyncWrite + Send + Unpin),
//     ) -> std::result::Result<(), Self::Error> {
//         self.as_rpc_error().code().send_to(io).await?;
//         self.as_rpc_error().message().send_to(io).await?;
//         Ok(())
//     }
// }

pub trait RpcError: Display + Debug + Send + Sync + 'static {
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

impl<Src, Dst> From<zerocopy::TryReadError<Src, Dst>> for Error
where
    Dst: ?Sized + TryFromBytes,
{
    fn from(err: zerocopy::TryReadError<Src, Dst>) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, format!("{:?}", err)).into()
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
