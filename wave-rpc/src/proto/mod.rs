use crate::{
    codec::{CodecRead, CodecWrite},
    Service,
};
use zerocopy::IntoBytes;

pub trait Proto<S: Service>:
    CodecRead<S::Request> + CodecRead<S::Response> + CodecWrite<S::Request> + CodecWrite<S::Response>
{
}

#[derive(Debug, IntoBytes, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C)]
pub struct Header {
    pub service_id: u32,
}
