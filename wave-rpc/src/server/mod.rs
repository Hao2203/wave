use crate::{
    codec::{CodecRead, CodecWrite},
    service::{Handle, Service},
};
use anyhow::Result;
use futures::future::BoxFuture;
use std::{collections::HashMap, hash::Hash};
use tokio::io::{AsyncRead, AsyncWrite};

pub mod transport;

pub struct RpcServer<'a, K, T, Conn> {
    map: HashMap<K, Box<dyn Handle<Conn> + 'a>>,
    transport: T,
}

impl<'a, K, T, Conn> RpcServer<'a, K, T, Conn> {
    pub fn new(transport: T) -> Self {
        Self {
            map: HashMap::new(),
            transport,
        }
    }
}

pub struct ConnHandler<'a, S, Codec> {
    service: &'a S,
    codec: &'a Codec,
}

impl<'a, S, Codec> ConnHandler<'a, S, Codec> {
    pub fn new(service: &'a S, codec: &'a Codec) -> Self {
        Self { service, codec }
    }
}
