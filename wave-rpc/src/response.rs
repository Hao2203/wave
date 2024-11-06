use crate::{
    body_stream::Body,
    error::{Code, Error, Result},
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
