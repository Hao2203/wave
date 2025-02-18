use std::{collections::HashMap, sync::Arc};

use crate::{Error, Host, Subdomain};

#[derive(Debug, Clone)]
pub struct RouterBuilder {
    map: HashMap<Subdomain, Host>,
}

impl RouterBuilder {
    pub fn new() -> RouterBuilder {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add(mut self, subdomain: Subdomain, ip: Host) -> Self {
        self.map.insert(subdomain, ip);
        self
    }

    pub fn build(mut self) -> Result<Router, Error> {
        if self.map.is_empty() {
            self.map.insert("".parse()?, "127.0.0.1".parse()?);
        }
        Ok(Router {
            inner: Arc::new(self.map),
        })
    }
}

impl Default for RouterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
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
