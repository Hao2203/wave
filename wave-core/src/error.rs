use crate::{NodeIdParsingError, WavePacketDecodeError};
use derive_more::{Display, From};

#[derive(Debug, From, Display, derive_more::Error)]
pub enum Error {
    #[from]
    NodeIdParsingError(NodeIdParsingError),
    #[from]
    WavePacketDecodeError(WavePacketDecodeError),
}
