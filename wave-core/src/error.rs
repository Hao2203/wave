use crate::{NodeIdParsingError, connection::WavePacketDecodeError};
use derive_more::{Display, From};
use std::sync::Arc;

#[derive(Debug, From, Display, derive_more::Error)]
pub enum Error {
    #[from]
    NodeIdParsingError(NodeIdParsingError),
    #[from]
    WavePacketDecodeError(WavePacketDecodeError),
    #[display("Subdomain overflow: {_0}")]
    #[error(ignore)]
    SubdomainOverflow(Arc<str>),
}
