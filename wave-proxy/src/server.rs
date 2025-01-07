use super::*;
use std::sync::Arc;

#[derive(Default)]
pub struct Builder<T> {
    proxies: Vec<Arc<dyn Proxy<T> + Send + Sync>>,
}

impl<T> Builder<T> {
    pub fn new() -> Self {
        Self {
            proxies: Vec::new(),
        }
    }

    pub fn add_proxy(mut self, proxy: impl Proxy<T> + Send + Sync + 'static) -> Self {
        self.proxies.push(Arc::new(proxy));
        self
    }

    pub fn build(self) -> Result<ProxyServer<T>> {
        Ok(ProxyServer {
            proxies: self.proxies.into(),
        })
    }
}

#[derive(Default)]
pub struct ProxyServer<T> {
    proxies: Arc<[Arc<dyn Proxy<T> + Send + Sync>]>,
}

impl<T> ProxyServer<T> {
    pub fn builder() -> Builder<T> {
        Builder::new()
    }
}

impl<T> ProxyServer<T>
where
    T: AsyncRead + AsyncWrite + Send + Unpin,
{
    pub async fn serve<'a>(&self, mut conn: T, local_addr: SocketAddr) -> Result<ProxyHandler> {
        let mut buf = bytes::BytesMut::with_capacity(1024);
        conn.read_buf(&mut buf).await.unwrap();
        let buf = buf.freeze();
        let io_buf = Cursor::new(buf);

        for proxy in self.proxies.iter() {
            let incoming = Incoming {
                io_buf: io_buf.clone(),
                conn,
                local_addr,
            };

            match proxy.serve(incoming).await? {
                ProxyStatus::Success(info) => return Ok(info),
                ProxyStatus::Continue(io) => conn = io,
            }
        }

        Err(Error::new(
            ErrorInner::UnSupportedProxyProtocol,
            "Unsupported proxy protocol in mixed proxy",
        ))
    }
}
