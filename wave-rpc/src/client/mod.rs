#![allow(unused)]
use crate::{
    body::BodyCodec,
    request::RequestEncoder,
    response::{ResponseDecoder, ResponseEncoder},
    service::Version,
    Request, Response, Service,
};
use error::{ClientError, Result};
use futures::{Sink, SinkExt, Stream, StreamExt};
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

/// ```rust
/// use wave_rpc::client::RpcClient;
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
/// let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
/// let client = RpcClient::new(1024 * 1024 * 10);
/// let conn = TcpStream::connect("127.0.0.1:8080").await.unwrap();
///
/// let req = AddReq(1, 2);
/// let res = client.connect_to::<MyService>(conn).await.unwrap().call(req).await.unwrap();
///
/// ```
pub struct RpcClient {
    max_body_size: usize,
    version: Version,
}

impl RpcClient {
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

    pub async fn connect_to<S>(
        &self,
        io: impl AsyncRead + AsyncWrite + Send + Sync + Unpin,
    ) -> Result<impl Call<S>>
    where
        S: Service,
        <S as Service>::Request: Serialize + Send,
        <S as Service>::Response: for<'a> Deserialize<'a> + Send,
    {
        let body_codec = BodyCodec::new(self.max_body_size);
        let response_decoder = ResponseDecoder::new(body_codec);
        let request_encoder = RequestEncoder::new(response_decoder);
        let framed = Framed::new(io, request_encoder);

        Ok(Caller {
            io: framed,
            service_version: self.version,
        })
    }
}

pub struct Caller<T> {
    io: T,
    service_version: Version,
}

impl<T, S> Call<S> for Caller<T>
where
    S: Service,
    <S as Service>::Request: Serialize + Send,
    <S as Service>::Response: for<'a> Deserialize<'a> + Send,
    T: Stream<Item = Result<Response, crate::error::Error>>
        + Sink<Request, Error = crate::error::Error>
        + Send
        + Sync
        + Unpin,
{
    async fn call(&mut self, req: <S as Service>::Request) -> Result<<S as Service>::Response> {
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
