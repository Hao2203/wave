use crate::{Connection, Error, NodeId, Subdomain, WavePacket};
use bytes::{Bytes, BytesMut};
use http::Response;
use std::{collections::HashMap, net::IpAddr, str::FromStr, sync::Arc};

#[derive(Debug, Default, Clone)]
pub struct Server {
    router: HashMap<Subdomain, IpAddr>,
}

impl Server {
    pub fn new(router: HashMap<Subdomain, IpAddr>) -> Self {
        Self { router }
    }

    pub fn try_from_iter(iter: impl IntoIterator<Item = (String, String)>) -> Result<Self, Error> {
        let router = iter
            .into_iter()
            .map(|(subdomain, ipaddr)| {
                let subdomain = Subdomain::new(Arc::from(subdomain))?;
                let ipaddr = IpAddr::from_str(&ipaddr)?;
                Ok((subdomain, ipaddr))
            })
            .collect::<Result<_, Error>>()?;

        Ok(Self { router })
    }

    pub fn add(&mut self, subdomain: Subdomain, ip: IpAddr) {
        self.router.insert(subdomain, ip);
    }

    pub fn accept(
        &self,
        node_id: NodeId,
        packet: WavePacket,
    ) -> (Connection, Result<IpAddr, Fallback>) {
        let conn = Connection::accept(node_id, packet);
        let ip = self
            .router
            .get(&conn.subdomain())
            .copied()
            .ok_or_else(Fallback::default);

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
