use std::{collections::HashMap, sync::Arc};

use crate::{Host, Subdomain};

#[derive(Debug, Default, Clone)]
pub struct RouterBuilder {
    map: HashMap<Subdomain, Host>,
}

impl RouterBuilder {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add(mut self, subdomain: Subdomain, ip: Host) -> Self {
        self.map.insert(subdomain, ip);
        self
    }

    pub fn build(self) -> Router {
        Router {
            inner: Arc::new(self.map),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Router {
    inner: Arc<HashMap<Subdomain, Host>>,
}

impl Router {
    pub fn builder() -> RouterBuilder {
        RouterBuilder::new()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<Subdomain, Host> {
        self.inner.iter()
    }

    pub fn find_host(&self, subdomain: &Subdomain) -> Option<Host> {
        self.inner.get(subdomain).cloned()
    }
}

impl<'a> IntoIterator for &'a Router {
    type Item = (&'a Subdomain, &'a Host);
    type IntoIter = std::collections::hash_map::Iter<'a, Subdomain, Host>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}
