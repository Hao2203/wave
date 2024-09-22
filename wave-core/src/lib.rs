use anyhow::Result;

pub mod author;
pub mod node;
pub mod resource;
pub mod store;
pub mod topic;

pub use store::{KVStore, MakeStore};

#[cfg(test)]
mod test;
