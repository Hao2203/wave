use crate::Body;

pub struct Response<'conn> {
    body: Body<'conn>,
}

impl<'conn> Response<'conn> {
    pub fn new(body: Body<'conn>) -> Self {
        Self { body }
    }

    pub fn body(&self) -> &Body<'conn> {
        &self.body
    }

    pub fn into_body(self) -> Body<'conn> {
        self.body
    }
}
