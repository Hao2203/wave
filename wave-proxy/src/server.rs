use std::sync::Arc;

use error::Context;
use tokio::net::TcpListener;

use super::*;
pub struct ProxyServer {
    listener: TcpListener,
    proxy: Arc<dyn Proxy>,
}

impl ProxyServer {
    pub fn builder<T: Proxy>(proxy: T) -> Builder<T> {
        Builder::new(proxy)
    }

    pub async fn accept(&self) -> Result<Incoming> {
        let (io, source_addr) = self.listener.accept().await.context("accept failed")?;
        let proxy = self.proxy.clone();
        Ok(Incoming {
            proxy,
            io: Box::new(io),
            source_addr,
        })
    }
}

pub struct Builder<T> {
    addrs: Vec<SocketAddr>,
    proxy: T,
}

impl<T> Builder<T> {
    pub fn new(proxy: T) -> Self {
        Self {
            addrs: Vec::new(),
            proxy,
        }
    }

    pub fn bind(mut self, addr: SocketAddr) -> Self {
        self.addrs.push(addr);
        self
    }
}

impl<T> Builder<T>
where
    T: Proxy + 'static,
{
    pub async fn build(self) -> Result<ProxyServer> {
        let addrs = self.addrs.as_slice();
        let listener = TcpListener::bind(addrs).await.context("bind failed")?;
        let proxy = Arc::new(self.proxy);
        Ok(ProxyServer { listener, proxy })
    }
}

pub struct ProxyConnection {
    pub target: Target,
    pub io: Box<dyn Io>,
    pub source_addr: SocketAddr,
}

pub struct Incoming {
    proxy: Arc<dyn Proxy>,
    io: Box<dyn Io>,
    source_addr: SocketAddr,
}

impl Incoming {
    // pub async fn start_proxy(self) -> Result<ProxyConnection> {
    //     let (info, io) = self.proxy.proxy(self.io).await?;
    //     Ok(ProxyConnection {
    //         target: info.target,
    //         io,
    //         source_addr: self.source_addr,
    //     })
    // }
}
