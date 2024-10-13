use crate::body::Body;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub struct Request<'conn> {
    pub header: &'conn Header,
    pub body: Body<'conn>,
}

impl<'conn> Request<'conn> {
    pub fn header(&self) -> &Header {
        self.header
    }

    pub fn body(&self) -> &Body<'conn> {
        &self.body
    }
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Header {
    pub req_id: u64,
    pub service: [u8; 32],
}
