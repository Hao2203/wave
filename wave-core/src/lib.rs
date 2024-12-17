use iroh::PublicKey;

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
pub struct Address(PublicKey);
