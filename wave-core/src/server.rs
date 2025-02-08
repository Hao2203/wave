use crate::{Connection, NodeId, Subdomain, WavePacket};
use std::{collections::HashMap, net::IpAddr};

#[derive(Debug, Default, Clone)]
pub struct Server {
    router: HashMap<Subdomain, IpAddr>,
}

impl Server {
    pub fn new(router: HashMap<Subdomain, IpAddr>) -> Self {
        Self { router }
    }

    pub fn add(&mut self, subdomain: Subdomain, ip: IpAddr) {
        self.router.insert(subdomain, ip);
    }

    pub fn accept(&self, node_id: NodeId, packet: WavePacket) -> (Connection, Option<IpAddr>) {
        let conn = Connection::accept(node_id, packet);
        let ip = self.router.get(&conn.subdomain()).copied();
        (conn, ip)
    }
}

impl IntoIterator for Server {
    type Item = (Subdomain, IpAddr);
    type IntoIter = std::collections::hash_map::IntoIter<Subdomain, IpAddr>;

    fn into_iter(self) -> Self::IntoIter {
        self.router.into_iter()
    }
}

impl<'a> IntoIterator for &'a Server {
    type Item = (&'a Subdomain, &'a IpAddr);
    type IntoIter = std::collections::hash_map::Iter<'a, Subdomain, IpAddr>;

    fn into_iter(self) -> Self::IntoIter {
        self.router.iter()
    }
}
