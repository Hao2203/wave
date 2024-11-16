#![allow(unused)]
use crate::error::{Error, Result};

pub struct Response<T> {
    code: u16,
    body: T,
}

impl<T> Response<T> {
    pub const CODE_SIZE: usize = 2;
    pub const SUCCESS_CODE: u16 = 0;

    pub fn is_success(&self) -> bool {
        self.code == Self::SUCCESS_CODE
    }

    pub fn code(&self) -> u16 {
        self.code
    }
}

// impl<'a> Transport for Response<'a> {
//     type Error = Error;

//     async fn from_reader(
//         mut reader: impl AsyncRead + Send + Sync + Unpin + 'a,
//     ) -> Result<Option<Self>, Self::Error>
//     where
//         Self: Sized,
//     {
//         let code = reader.read_u16_le().await?;
//         let body = Body::from_reader(reader).await?;
//         let resp = body.map(|body| Response::new(code, body));
//         Ok(resp)
//     }

//     async fn write_into(
//         &mut self,
//         mut io: impl AsyncWrite + Send + Unpin,
//     ) -> Result<(), Self::Error> {
//         io.write_u16_le(self.code).await?;
//         self.body.write_into(io).await?;
//         Ok(())
//     }
// }
