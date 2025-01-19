use anyhow::anyhow;

use super::*;
use std::sync::Arc;

#[derive(Default)]
pub struct Builder<T> {
    proxies: Vec<Arc<dyn ProxyService<T> + Send + Sync>>,
    app: T,
}

impl<T> Builder<T> {
    pub fn new(app: T) -> Self {
        Self {
            proxies: Vec::new(),
            app,
        }
    }

    pub fn add_proxy(mut self, service: impl ProxyService<T> + Send + Sync + 'static) -> Self
    where
        T: ProxyApp,
    {
        self.proxies.push(Arc::new(service));
        self
    }

    pub fn build(self) -> Result<ProxyServer<T>> {
        Ok(ProxyServer {
            proxies: self.proxies.into(),
            app: self.app,
        })
    }
}

#[derive(Default, Clone)]
pub struct ProxyServer<T> {
    proxies: Arc<[Arc<dyn ProxyService<T> + Send + Sync>]>,
    app: T,
}

impl<T> ProxyServer<T> {
    pub fn builder(app: T) -> Builder<T> {
        Builder::new(app)
    }
}

impl<T> ProxyServer<T>
where
    T: ProxyApp,
{
    pub async fn serve<'a>(
        &self,
        mut conn: impl Connection + Unpin,
        local_addr: SocketAddr,
    ) -> Result<()> {
        let mut buf = bytes::BytesMut::with_capacity(1024);
        conn.read_buf(&mut buf).await.unwrap();
        let buf = buf.freeze();
        let io_buf = Cursor::new(buf);

        let mut incoming = Incoming {
            io_buf,
            conn: &mut conn,
            local_addr,
        };

        for proxy in self.proxies.iter() {
            incoming =
                if let ProxyStatus::Continue(incoming) = proxy.serve(&self.app, incoming).await? {
                    incoming
                } else {
                    return Ok(());
                }
        }

        Err(Error::new(
            ErrorKind::UnSupportedProxyProtocol,
            Some(anyhow!("Unsupported proxy protocol")),
        ))
    }
}
