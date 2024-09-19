use crate::{
    author::Author,
    resource::{MakeResourceKind, Resource},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use ulid::Ulid;

pub mod store;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Message<T> {
    id: Ulid,
    data: T,
    author: Author,
    timestamp: u64,
    resource: Option<Resource>,
}

impl<T> Message<T> {
    pub fn new(data: T, author: Author, timestamp: u64, resource: Option<Resource>) -> Self {
        let id = Ulid::new();
        Self {
            id,
            data,
            author,
            timestamp,
            resource,
        }
    }

    pub fn id(&self) -> u128 {
        self.id.into()
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn into_data(self) -> T {
        self.data
    }

    pub fn author(&self) -> &Author {
        &self.author
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn resource(&self) -> Option<&Resource> {
        self.resource.as_ref()
    }
}

pub trait MakeResource {
    fn make_resource(&self) -> Option<MakeResourceKind>;
}
