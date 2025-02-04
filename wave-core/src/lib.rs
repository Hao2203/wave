use derive_more::{Display, From};
use iroh::{KeyParsingError, PublicKey};
use std::str::FromStr;

pub mod server;
#[cfg(test)]
mod test;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    derive_more::From,
)]
pub struct NodeId(PublicKey);

impl FromStr for NodeId {
    type Err = KeyParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let public_key = PublicKey::from_str(s)?;
        Ok(NodeId(public_key))
    }
}

#[derive(Debug, From, Display, derive_more::Error)]
pub enum Error {
    NodeIdParsingError(KeyParsingError),
}
