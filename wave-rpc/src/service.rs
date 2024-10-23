use serde::{Deserialize, Serialize};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub trait Service {
    type Request;
    type Response;

    const ID: u32;
}

#[derive(
    Debug,
    Copy,
    Clone,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    IntoBytes,
    FromBytes,
    KnownLayout,
    Immutable,
    Serialize,
    Deserialize,
)]
#[repr(transparent)]
pub struct Version(u32);

impl Version {
    pub const fn new(version: u32) -> Self {
        Self(version)
    }
    pub fn inner(&self) -> u32 {
        self.0
    }
}

impl AsRef<u32> for Version {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl From<u32> for Version {
    fn from(version: u32) -> Self {
        Self::new(version)
    }
}

impl From<Version> for u32 {
    fn from(version: Version) -> Self {
        version.0
    }
}
