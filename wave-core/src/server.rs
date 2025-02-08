use crate::{Connection, Error, NodeId, Subdomain, WavePacket};
use bytes::{Bytes, BytesMut};
use derive_more::Display;
use http::Response;
use std::{collections::HashMap, net::IpAddr, str::FromStr, sync::Arc};

#[derive(Debug, Clone, Display)]
pub enum Host {
    Ip(IpAddr),
    Domain(Arc<str>),
}

impl Host {
    pub const MAX_LEN: usize = 255;
}

impl FromStr for Host {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > Host::MAX_LEN {
            return Err(Error::DomainOverflow(Arc::from(s)));
        }
        if let Ok(ip) = s.parse() {
            Ok(Host::Ip(ip))
        } else {
            Ok(Host::Domain(Arc::from(s)))
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Server {
    router: HashMap<Subdomain, Host>,
}

impl Server {
    pub fn new(router: HashMap<Subdomain, Host>) -> Self {
        Self { router }
    }

    pub fn try_from_iter(iter: impl IntoIterator<Item = (String, String)>) -> Result<Self, Error> {
        let router = iter
            .into_iter()
            .map(|(subdomain, ipaddr)| {
                let subdomain = Subdomain::new(Arc::from(subdomain))?;
                let host = Host::from_str(&ipaddr)?;
                Ok((subdomain, host))
            })
            .collect::<Result<_, Error>>()?;

        Ok(Self { router })
    }

    pub fn add(&mut self, subdomain: Subdomain, ip: Host) {
        self.router.insert(subdomain, ip);
    }

    pub fn accept(
        &self,
        node_id: NodeId,
        packet: WavePacket,
    ) -> (Connection, Result<Host, Fallback>) {
        let conn = Connection::accept(node_id, packet);
        let ip = self
            .router
            .get(&conn.subdomain())
            .cloned()
            .ok_or_else(Fallback::default);

        (conn, ip)
    }
}

impl IntoIterator for Server {
    type Item = (Subdomain, Host);
    type IntoIter = std::collections::hash_map::IntoIter<Subdomain, Host>;

    fn into_iter(self) -> Self::IntoIter {
        self.router.into_iter()
    }
}

impl<'a> IntoIterator for &'a Server {
    type Item = (&'a Subdomain, &'a Host);
    type IntoIter = std::collections::hash_map::Iter<'a, Subdomain, Host>;

    fn into_iter(self) -> Self::IntoIter {
        self.router.iter()
    }
}

pub struct Fallback {
    data: Bytes,
}

impl Fallback {
    pub fn bytes(&self) -> Bytes {
        self.data.clone()
    }
}

const FALLBACK_HTML: &str = include_str!("../static/fallback.html");

impl Default for Fallback {
    fn default() -> Self {
        let response = Response::builder()
            .status(404)
            .header("Content-Type", "text/html")
            .body(Bytes::from(FALLBACK_HTML))
            .unwrap();
        let (data, body) = response.into_parts();
        let mut buf = BytesMut::new();
        buf.extend_from_slice(format!("HTTP/1.1 {}\r\n", data.status).as_bytes());
        for (key, value) in data.headers.iter() {
            buf.extend_from_slice(
                format!("{}: {}\r\n", key.as_str(), value.to_str().unwrap()).as_bytes(),
            );
        }
        buf.extend_from_slice(b"\r\n");
        buf.extend_from_slice(body.as_ref());

        Self {
            data: Bytes::from(FALLBACK_HTML),
        }
    }
}
