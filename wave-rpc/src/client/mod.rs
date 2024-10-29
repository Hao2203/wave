use crate::{
    body::BodyCodec, error::Error, message::Message, request::RequestEncoder,
    response::ResponseDecoder, service::Version, Request, Response, Service,
};
use error::{ClientError, Result};
use futures::{Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;
use tracing::{instrument, trace, Level};

pub mod error;
pub mod pool;

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

impl Builder {
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

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Client<'a> {
    io: Box<dyn Transport + Send + Sync + 'a>,
    service_version: Version,
}

impl<'a> Client<'a> {
    pub fn builder() -> Builder {
        Builder::new()
    }

    fn new<T>(io: T, service_version: Version) -> Self
    where
        for<'b> T: Stream<Item = Result<Response>>
            + Sink<&'b Request, Error = Error>
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

    #[instrument(skip_all, level = Level::TRACE, name = "client_call", err(level = Level::WARN))]
    pub async fn call<S>(
        &mut self,
        req: <S::Request as Message>::Inner,
    ) -> Result<<S::Response as Message>::Inner>
    where
        S: Service,
        <S as Service>::Request: Message + Send,
        <S as Service>::Response: Message + Send,
    {
        let req = Request::new::<S>(req, self.service_version)?;

        trace!(
            service_id = S::ID,
            service_version = self.service_version.inner(),
            "start call remote service",
        );

        self.io.send(&req).await?;
        self.io.flush().await?;
        let mut res = self
            .io
            .next()
            .await
            .ok_or(ClientError::ReceiveResponseFailed)??;

        if !res.is_success() {
            Err(ClientError::ErrorWithCode(res.code()))?;
        }
        let res = S::Response::from_body(res.body_mut()).unwrap();

        trace!(
            service_id = S::ID,
            service_version = self.service_version.inner(),
            "finish call remote service",
        );

        Ok(res)
    }
}

fn to_stream_and_sink(
    io: impl AsyncRead + AsyncWrite + Send + Sync + Unpin,
    body_codec: BodyCodec,
) -> impl Stream<Item = Result<Response>> + for<'a> Sink<&'a Request, Error = Error> + Unpin {
    let response_decoder = ResponseDecoder::new(body_codec);
    let request_encoder = RequestEncoder::new(response_decoder);

    Framed::new(io, request_encoder).map_err(From::from)
}

pub trait Transport:
    for<'a> Sink<&'a Request, Error = Error> + Stream<Item = Result<Response>> + Unpin
{
}

impl<T> Transport for T where
    for<'a> T: Sink<&'a Request, Error = Error> + Stream<Item = Result<Response>> + Unpin
{
}
