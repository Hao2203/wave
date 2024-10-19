use crate::{body::BodyCodec, Body};
use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

pub struct Response {
    body: Body,
}

impl Response {
    pub fn new(body: Body) -> Self {
        Self { body }
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
        let Response { body } = item;
        self.body_codec.encode(body, dst)?;
        Ok(())
    }
}

impl Decoder for ResponseCodec {
    type Item = Response;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let body = self.body_codec.decode(src)?;
        Ok(body.map(|body| Response { body }))
    }
}
