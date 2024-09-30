use crate::{
    codec::{CodecRead, CodecWrite},
    handle::{RefReader, RefWriter},
    service::Service,
    Handle,
};
use anyhow::Result;
use futures::future::BoxFuture;
use std::{collections::HashMap, hash::Hash, pin::Pin};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    pin,
};

pub struct RpcServer<'a, K, Codec> {
    map: HashMap<K, Box<dyn Handle<'a> + 'a>>,
    codec: Codec,
}

impl<'a, K, Codec> RpcServer<'a, K, Codec> {
    #[allow(clippy::new_without_default)]
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
    {
        self.map
            .insert(S::KEY, ConnHandler::boxed(service, &self.codec));
    }

    pub async fn serve(
        &'a self,
        mut reader: impl AsyncRead + Send + Unpin,
        writer: impl AsyncWrite + Send + Unpin,
    ) -> Result<()>
    where
        K: Eq + Hash + Send + 'static,
        Codec: CodecRead<K>,
    {
        let key = self.codec.codec_read(Pin::new(&mut reader)).await?;
        let handler = self.map.get(&key).unwrap();
        pin!(reader, writer);
        handler.handle(reader, writer).await?;
        Ok(())
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

    pub fn boxed(service: &'a S, codec: &'a Codec) -> Box<dyn Handle<'a> + 'a>
    where
        S: Service<Req> + Send + Sync + 'a,
        Req: Send + 'static,
        S::Response: Send + 'static,
        Codec: CodecRead<Req> + CodecWrite<S::Response> + Send + Sync + 'a,
    {
        Box::new(ConnHandler::new(service, codec))
    }
}

impl<'a, S, Req, Codec> Handle<'a> for ConnHandler<'a, S, Req, Codec>
where
    S: Service<Req> + Send + Sync + 'a,
    Req: Send + 'static,
    S::Response: Send + 'static,
    Codec: CodecRead<Req> + CodecWrite<S::Response> + Send + Sync + 'a,
{
    fn handle<'conn>(
        &'a self,
        reader: RefReader<'conn>,
        writer: RefWriter<'conn>,
    ) -> BoxFuture<'conn, Result<()>>
    where
        'a: 'conn,
    {
        let fut = async {
            let req = self.codec.codec_read(reader).await?;
            let res = self.service.call(req).await?;
            self.codec.codec_write(writer, res).await?;
            anyhow::Ok(())
        };
        Box::pin(fut)
    }
}
