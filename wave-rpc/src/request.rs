use crate::body::{Body, BodyType};
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

    pub fn body_mut(&mut self) -> &mut Body<'conn> {
        &mut self.body
    }
}

#[derive(TryFromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C, packed)]
pub struct Header {
    pub service_id: u64,
    pub body_type: BodyType,
    pub body_size: u64, // if body_type == BodyType::Bytes then this is the size in bytes else it's the stream item length
}
