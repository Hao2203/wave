use crate::body::Body;
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

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

#[derive(TryFromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C, packed)]
pub struct Header {
    pub req_id: u64,
    pub body_type: BodyType,
    pub body_size: u64, // if body_type == BodyType::Bytes then this is the size in bytes else it's the stream item length
}

#[derive(IntoBytes, TryFromBytes, KnownLayout, Immutable, Unaligned)]
#[repr(u8)]
pub enum BodyType {
    Bytes,
    Stream,
}
