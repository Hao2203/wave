use crate::body::Body;

pub struct Request<'conn, const N: usize = 4> {
    header: [u8; N],
    body: Body<'conn>,
}

impl<'conn, const N: usize> Request<'conn, N> {
    pub fn new(header: [u8; N], body: Body<'conn>) -> Self {
        Self { header, body }
    }

    pub fn header(&self) -> &[u8; N] {
        &self.header
    }

    pub fn body(&self) -> &Body<'conn> {
        &self.body
    }
}
