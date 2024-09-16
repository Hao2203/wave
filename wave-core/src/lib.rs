use serde::{de::DeserializeOwned, Serialize};

pub mod node;

pub trait Entity {
    type Id: AsRef<[u8; 32]>;
    type Data: Serialize + DeserializeOwned + Send + Sync;
}
