use serde::{Deserialize, Serialize};
use std::fmt::Display;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::message::Message;

pub trait Service {
    type Request<'a>;
    type Response<'a>;

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

    #[inline]
    pub const fn inner(&self) -> u32 {
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

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
