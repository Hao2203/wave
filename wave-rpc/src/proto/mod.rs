use crate::{
    service::{Connection, Handle},
    Service,
};
use zerocopy::IntoBytes;

pub trait Proto<S: Service> {
    fn client(&self, conn: &mut dyn Connection) -> S;

    fn server(&self, service: S) -> Box<dyn Handle<dyn Connection>>;
}

#[derive(Debug, IntoBytes, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C)]
pub struct Header {
    pub service_id: u32,
}
