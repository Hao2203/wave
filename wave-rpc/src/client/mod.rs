#![allow(unused)]
use crate::{
    body::BodyCodec,
    error::Error,
    request::RequestEncoder,
    response::{ResponseDecoder, ResponseEncoder},
    service::Version,
    Request, Response, Service,
};
use error::{ClientError, Result};
use futures::{Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

pub mod error;
pub mod pool;

pub trait Call<S: Service> {
    fn call(
        &mut self,
        req: S::Request,
    ) -> impl std::future::Future<Output = Result<S::Response>> + Send;
}

/// ```no_run
/// use wave_rpc::client::RpcBuilder;
/// use wave_rpc::service::Service;
/// use wave_rpc::server::RpcService;
/// use tokio::net::{TcpStream, TcpListener};
///
/// struct MyService;
///
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct AddReq(u32, u32);
///
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct AddRes(u32);
///
/// impl Service for MyService {
///     type Request = AddReq;
///     type Response = AddRes;
///
///     const ID: u32 = 1;
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let conn = TcpStream::connect("127.0.0.1:8080").await.unwrap();
///     let builder = RpcBuilder::new(1024 * 1024 * 10);
///     let mut client = builder.build_client(conn).await.unwrap();
///     let req = AddReq(1, 2);
///     let res = client.call::<MyService>(req).await.unwrap();
/// }
///
/// ```
pub struct RpcBuilder {
    max_body_size: usize,
    version: Version,
}

impl RpcBuilder {
    pub fn new(max_body_size: usize) -> Self {
        Self {
            max_body_size,
            version: Default::default(),
        }
    }

    pub fn version(mut self, version: impl Into<Version>) -> Self {
        self.version = version.into();
        self
    }

    pub fn max_body_size(mut self, max_body_size: usize) -> Self {
        self.max_body_size = max_body_size;
        self
    }

    pub async fn build_client(
        &self,
        io: impl AsyncRead + AsyncWrite + Send + Sync + Unpin,
    ) -> Result<Client<impl Stream<Item = Result<Response>> + Sink<Request, Error = Error> + Unpin>>
    {
        let body_codec = BodyCodec::new(self.max_body_size);
        let response_decoder = ResponseDecoder::new(body_codec);
        let request_encoder = RequestEncoder::new(response_decoder);
        let framed = Framed::new(io, request_encoder).map_err(From::from);

        Ok(Client {
            io: framed,
            service_version: self.version,
        })
    }
}

pub struct Client<T> {
    io: T,
    service_version: Version,
}

impl<T> Client<T> {
    pub async fn call<S>(&mut self, req: S::Request) -> Result<S::Response>
    where
        S: Service,
        <S as Service>::Request: Serialize + Send,
        <S as Service>::Response: for<'a> Deserialize<'a> + Send,
        T: Stream<Item = Result<Response>> + Sink<Request, Error = Error> + Unpin,
    {
        let req = Request::new::<S>(req, self.service_version)?;
        self.io.send(req).await?;
        let res = self
            .io
            .next()
            .await
            .ok_or_else(|| ClientError::ReceiveResponseFailed)??;
        if !res.is_success() {
            Err(ClientError::ErrorWithCode(res.code()))?;
        }
        let res = res.into_body().bincode_decode()?;
        Ok(res)
    }
}
