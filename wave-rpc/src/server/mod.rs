use crate::{
    codec::{CodecRead, CodecWrite},
    service::Service,
};
use anyhow::Result;
use futures::future::BoxFuture;
use std::{collections::HashMap, hash::Hash};
use tokio::io::{AsyncRead, AsyncWrite};

pub mod router;

pub trait Handle<'a, Conn> {
    fn handle<'conn>(&'a self, conn: &'conn mut Conn) -> BoxFuture<'conn, anyhow::Result<()>>
    where
        'a: 'conn,
        Conn: AsyncRead + AsyncWrite + Unpin;
}

pub struct RpcServer<'a, K, Codec, Conn> {
    map: HashMap<K, Box<dyn Handle<'a, Conn> + 'a>>,
    codec: Codec,
}

impl<'a, K, Codec, Conn> RpcServer<'a, K, Codec, Conn> {
    pub fn new(codec: Codec) -> Self {
        Self {
            map: HashMap::new(),
            codec,
        }
    }

    pub fn register<S, Req>(&'a mut self, service: &'a S)
    where
        S: Service<Req, Key = K> + Send + Sync + 'a,
        K: Eq + Hash + Send,
        Req: Send + 'static,
        S::Response: Send + 'static,
        Codec: CodecRead<Req> + CodecWrite<S::Response> + Send + Sync + 'a,
        Conn: AsyncRead + AsyncWrite + Unpin + Send,
    {
        self.map
            .insert(service.key(), ConnHandler::boxed(service, &self.codec));
    }
}

pub struct ConnHandler<'a, S, Req, Codec> {
    service: &'a S,
    codec: &'a Codec,
    _req: std::marker::PhantomData<Req>,
}

impl<'a, S, Req, Codec> ConnHandler<'a, S, Req, Codec> {
    pub fn new(service: &'a S, codec: &'a Codec) -> Self {
        Self {
            service,
            codec,
            _req: std::marker::PhantomData,
        }
    }

    pub fn boxed<Conn>(service: &'a S, codec: &'a Codec) -> Box<dyn Handle<'a, Conn> + 'a>
    where
        S: Service<Req> + Send + Sync + 'a,
        Req: Send + 'static,
        S::Response: Send + 'static,
        Codec: CodecRead<Req> + CodecWrite<S::Response> + Send + Sync + 'a,
        Conn: AsyncRead + AsyncWrite + Unpin + Send,
    {
        Box::new(ConnHandler::new(service, codec))
    }
}

impl<'a, S, Req, Codec, Conn> Handle<'a, Conn> for ConnHandler<'a, S, Req, Codec>
where
    S: Service<Req> + Send + Sync + 'a,
    Req: Send + 'static,
    S::Response: Send + 'static,
    Codec: CodecRead<Req> + CodecWrite<S::Response> + Send + Sync + 'a,
    Conn: AsyncRead + AsyncWrite + Unpin + Send,
{
    fn handle<'conn>(&'a self, conn: &'conn mut Conn) -> BoxFuture<'conn, Result<()>>
    where
        'a: 'conn,
    {
        let fut = async {
            let req = self.codec.codec_read(conn).await?;
            let res = self.service.call(req).await?;
            self.codec.codec_write(conn, res).await?;
            anyhow::Ok(())
        };
        Box::pin(fut)
    }
}
