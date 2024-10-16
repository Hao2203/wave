use crate::{body::BodyType, Body};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

pub struct Response<'conn> {
    header: Header,
    body: Body<'conn>,
}

impl<'conn> Response<'conn> {
    pub fn new(header: Header, body: Body<'conn>) -> Self {
        Self { header, body }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn body(&self) -> &Body<'conn> {
        &self.body
    }

    pub fn into_body(self) -> Body<'conn> {
        self.body
    }

    pub fn body_mut(&mut self) -> &mut Body<'conn> {
        &mut self.body
    }
}

#[derive(TryFromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C, packed)]
pub struct Header {
    pub body_type: BodyType,
    pub body_size: u64,
}

impl Header {
    pub const SIZE: usize = 9;

    #[inline]
    pub fn buffer() -> [u8; Self::SIZE] {
        [0u8; Self::SIZE]
    }

    pub fn as_bytes(&self) -> &[u8] {
        <Self as IntoBytes>::as_bytes(self)
    }
}
