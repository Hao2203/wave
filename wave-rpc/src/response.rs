use crate::{
    error::{Error, ErrorCode, Result},
    Body,
};
use bytes::{Buf, BufMut, BytesMut};
use serde::Serialize;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone)]
pub struct Response {
    code: u16,
    body: Body,
}

impl Response {
    pub const CODE_SIZE: usize = 2;
    pub const SUCCESS_CODE: u16 = 0;

    pub fn new(code: u16, body: Body) -> Self {
        Self { body, code }
    }

    pub fn success(body: Body) -> Self {
        Self::new(0, body)
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn is_success(&self) -> bool {
        self.code == Self::SUCCESS_CODE
    }

    pub fn error_code(&self) -> Option<ErrorCode> {
        if self.is_success() {
            None
        } else {
            Some(ErrorCode::from(self.code))
        }
    }

    pub fn code(&self) -> u16 {
        self.code
    }

    pub fn into_body(self) -> Body {
        self.body
    }

    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }
}

impl<T> TryFrom<Result<T>> for Response
where
    T: Serialize,
{
    type Error = Error;
    fn try_from(result: Result<T>) -> Result<Self, Self::Error> {
        let res = result
            .and_then(|value| Body::bincode_encode(&value))
            .map(Response::success);
        res.try_into()
    }
}

impl TryFrom<Result<Response>> for Response {
    type Error = Error;
    fn try_from(result: Result<Response>) -> Result<Self, Self::Error> {
        Ok(match result {
            Ok(response) => response,
            Err(err) => {
                let code = ErrorCode::try_from(err)?;
                Self::new(code.into(), Body::new_empty())
            }
        })
    }
}

pub(crate) struct ResponseDecoder<T> {
    codec: T,
}

impl<T> ResponseDecoder<T> {
    pub fn new(codec: T) -> Self {
        Self { codec }
    }
}

impl<T> Decoder for ResponseDecoder<T>
where
    T: Decoder<Item = Body, Error = Error>,
{
    type Item = Response;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < Response::CODE_SIZE {
            return Ok(None);
        }
        let code = src.get_u16();

        let body = self.codec.decode(src)?;
        Ok(body.map(|body| Response { code, body }))
    }
}

impl<T, B> Encoder<B> for ResponseDecoder<T>
where
    T: Encoder<B, Error = Error>,
{
    type Error = Error;

    fn encode(&mut self, item: B, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.codec.encode(item, dst)
    }
}

pub(crate) struct ResponseEncoder<T> {
    codec: T,
}

impl<T> ResponseEncoder<T> {
    pub fn new(codec: T) -> Self {
        Self { codec }
    }
}

impl<T> Decoder for ResponseEncoder<T>
where
    T: Decoder,
{
    type Error = T::Error;
    type Item = T::Item;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> std::result::Result<Option<Self::Item>, Self::Error> {
        self.codec.decode(src)
    }
}

impl<T> Encoder<Response> for ResponseEncoder<T>
where
    T: for<'a> Encoder<&'a Body, Error = Error>,
{
    type Error = Error;

    fn encode(&mut self, item: Response, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.encode(&item, dst)
    }
}

impl<T> Encoder<&Response> for ResponseEncoder<T>
where
    T: for<'a> Encoder<&'a Body, Error = Error>,
{
    type Error = Error;

    fn encode(&mut self, item: &Response, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let Response { body, code } = item;
        dst.reserve(Response::CODE_SIZE);
        dst.put_u16(*code);
        self.codec.encode(body, dst)?;
        Ok(())
    }
}

impl<T> Encoder<Body> for ResponseEncoder<T>
where
    T: Encoder<Body, Error = Error>,
{
    type Error = Error;

    fn encode(&mut self, item: Body, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.codec.encode(item, dst)
    }
}

impl<T> Encoder<&Body> for ResponseEncoder<T>
where
    T: for<'a> Encoder<&'a Body, Error = Error>,
{
    type Error = Error;

    fn encode(&mut self, item: &Body, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.codec.encode(item, dst)
    }
}
