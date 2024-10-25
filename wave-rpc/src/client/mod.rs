use crate::{
    body::BodyCodec, error::Error, request::RequestEncoder, response::ResponseDecoder,
    service::Version, Request, Response, Service,
};
use error::{ClientError, Result};
use futures::{Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

pub mod error;
pub mod pool;

/// # Example
///
/// ```no_run
/// use wave_rpc::client::Builder;
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
///     let builder = Builder::new();
///     let mut client = builder.build_client(conn).await.unwrap();
///     let req = AddReq(1, 2);
///     let res = client.call::<MyService>(req).await.unwrap();
/// }
///
/// ```
pub struct Builder<T = ()> {
    max_body_size: Option<usize>,
    version: Version,
    manager: T,
}

pub const DEFAULT_MAX_BODY_SIZE: usize = 1024 * 1024 * 10;

impl<T> Builder<T> {
    pub fn version(mut self, version: impl Into<Version>) -> Self {
        self.version = version.into();
        self
    }

    pub fn max_body_size(mut self, max_body_size: usize) -> Self {
        self.max_body_size = Some(max_body_size);
        self
    }
}

impl Builder<()> {
    pub fn new() -> Self {
        Self {
            max_body_size: None,
            version: Version::default(),
            manager: (),
        }
    }
    pub async fn build_client<'a>(
        &'a self,
        io: impl AsyncRead + AsyncWrite + Send + Sync + Unpin + 'a,
    ) -> Result<Client<'a>> {
        let body_codec = BodyCodec::new(self.max_body_size.unwrap_or(DEFAULT_MAX_BODY_SIZE));
        let io = to_stream_and_sink(io, body_codec);

        Ok(Client::new(io, self.version))
    }
}

impl Default for Builder<()> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Client<'a> {
    io: Box<dyn Transport + Send + Sync + 'a>,
    service_version: Version,
}

impl<'a> Client<'a> {
    fn new<T>(io: T, service_version: Version) -> Self
    where
        T: Stream<Item = Result<Response>>
            + Sink<Request, Error = Error>
            + Unpin
            + Send
            + Sync
            + 'a,
    {
        Self {
            io: Box::new(io),
            service_version,
        }
    }

    pub async fn call<S>(&mut self, req: S::Request) -> Result<S::Response>
    where
        S: Service,
        <S as Service>::Request: Serialize + Send,
        <S as Service>::Response: for<'b> Deserialize<'b> + Send,
    {
        let req = Request::new::<S>(req, self.service_version)?;

        println!("start call remote service");

        self.io.send(req).await?;
        self.io.flush().await?;
        let res = self
            .io
            .next()
            .await
            .ok_or(ClientError::ReceiveResponseFailed)??;

        println!("finish call remote service");

        if !res.is_success() {
            Err(ClientError::ErrorWithCode(res.code()))?;
        }
        let res = res.into_body().bincode_decode()?;
        Ok(res)
    }
}

fn to_stream_and_sink(
    io: impl AsyncRead + AsyncWrite + Send + Sync + Unpin,
    body_codec: BodyCodec,
) -> impl Stream<Item = Result<Response>> + Sink<Request, Error = Error> + Unpin {
    let response_decoder = ResponseDecoder::new(body_codec);
    let request_encoder = RequestEncoder::new(response_decoder);

    Framed::new(io, request_encoder).map_err(From::from)
}

// pub trait Call {
//     fn call<S>(&mut self, req: S::Request) -> impl Future<Output = Result<S::Response>> + Send
//     where
//         S: Service,
//         <S as Service>::Request: Serialize + Send,
//         <S as Service>::Response: for<'a> Deserialize<'a> + Send;
// }

// impl<T> Call for Client<T>
// where
//     T: Stream<Item = Result<Response>> + Sink<Request, Error = Error> + Unpin + Send,
// {
//     async fn call<S>(&mut self, req: S::Request) -> Result<S::Response>
//     where
//         S: Service,
//         <S as Service>::Request: Serialize + Send,
//         <S as Service>::Response: for<'a> Deserialize<'a> + Send,
//     {
//         self.call::<S>(req).await
//     }
// }

pub trait Transport:
    Stream<Item = Result<Response>> + Sink<Request, Error = Error> + Unpin
{
}

impl<T> Transport for T where
    T: Stream<Item = Result<Response>> + Sink<Request, Error = Error> + Unpin
{
}
