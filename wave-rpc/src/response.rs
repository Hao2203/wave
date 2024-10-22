use crate::{
    error::Result,
    {body::BodyCodec, Body},
};
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

pub struct Response {
    code: u16,
    body: Body,
}

impl Response {
    pub const CODE_LEN: usize = 2;
    pub fn new(code: u16, body: Body) -> Self {
        Self { body, code }
    }

    pub fn success(body: Body) -> Self {
        Self::new(0, body)
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn into_body(self) -> Body {
        self.body
    }

    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }
}

pub struct ResponseCodec {
    body_codec: BodyCodec,
}

impl ResponseCodec {
    pub fn new(body_codec: BodyCodec) -> Self {
        Self { body_codec }
    }
}

impl Encoder<Response> for ResponseCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Response, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let Response { body, code } = item;
        dst.reserve(Response::CODE_LEN);
        dst.put_u16(code);
        self.body_codec.encode(body, dst)?;
        Ok(())
    }
}

impl Decoder for ResponseCodec {
    type Item = Response;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < Response::CODE_LEN {
            return Ok(None);
        }
        let code = src.get_u16();

        let body = self.body_codec.decode(src)?;
        Ok(body.map(|body| Response { code, body }))
    }
}
