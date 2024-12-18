use derive_more::derive::{Display, Error, From};

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Display, From, Error)]
pub enum Error {}
