use crate::{Connection, Host, NodeId, Subdomain, WavePacket, router::Router};
use bytes::{Bytes, BytesMut};
use http::Response;

#[derive(Debug, Clone)]
pub struct Server {
    router: Router,
}

impl Server {
    pub fn new(router: Router) -> Self {
        Self { router }
    }

    pub fn accept(&self, remote_node_id: NodeId, packet: WavePacket) -> (Connection, Option<Host>) {
        let conn = Connection::accept(remote_node_id, packet);
        let ip = self.router.find_host(&conn.subdomain());

        (conn, ip)
    }

    pub fn get_target(&self, subdomain: &Subdomain) -> Option<Host> {
        self.router.find_host(subdomain)
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
