use crate::{
    body::BodyCodec,
    error::Result,
    request::{Request, RequestDecoder},
    response::{Response, ResponseEncoder},
};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
pub use service::RpcService;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

// pub mod error;
pub mod code;
pub mod service;

#[async_trait]
pub trait RpcHandler {
    async fn call(&self, req: &mut Request) -> Result<Response>;
}

/// # Example
/// ```no_run
/// use wave_rpc::server::RpcService;
/// use wave_rpc::service::Service;
/// use tokio::net::TcpListener;
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
/// struct MyServiceState;
///
/// impl MyServiceState {
///     async fn add(&self, req: AddReq) -> AddRes {
///         AddRes(req.0 + req.1)
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let service = RpcService::with_state(&MyServiceState).register::<MyService>(MyServiceState::add);
///     let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
///     let conn = listener.accept().await.unwrap().0;
///
///     let server = wave_rpc::server::RpcServer::new(1024);
///     server.serve(service, conn).await.unwrap();
/// }
///
/// ```
pub struct RpcServer {
    max_body_size: usize,
}

impl RpcServer {
    pub fn new(max_body_size: usize) -> Self {
        Self { max_body_size }
    }
    pub async fn serve(
        &self,
        service: impl RpcHandler,
        io: (impl AsyncRead + AsyncWrite + Send + Unpin),
    ) -> Result<()> {
        let body_codec = BodyCodec::new(self.max_body_size);
        let request_codec = RequestDecoder::new(body_codec);
        let response_codec = ResponseEncoder::new(request_codec);
        let framed = Framed::new(io, response_codec);
        let (mut sink, mut stream) = framed.split();

        while let Some(req) = stream.next().await {
            println!("start process request");

            let mut req = req?;
            let res = service.call(&mut req).await?;
            sink.send(res).await?;

            println!("finish process request");
        }

        Ok(())
    }
}
