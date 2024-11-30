use crate::{error::Error, message::SendTo};
use async_trait::async_trait;
use futures_lite::AsyncWrite;

pub struct Body<'a>(pub Box<dyn SendTo<Error = Error> + Send + 'a>);

impl<'a> Body<'a> {
    pub fn new(body: impl SendTo<Error: Into<Error>> + Send + 'a) -> Self {
        Self(Box::new(BodyInner(body)))
    }
}

#[async_trait]
impl SendTo for Body<'_> {
    type Error = Error;

    async fn send_to(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> std::result::Result<(), Self::Error> {
        self.0.send_to(io).await
    }
}

struct BodyInner<T>(T);

#[async_trait]
impl<T> SendTo for BodyInner<T>
where
    T: SendTo<Error: Into<Error>> + Send,
{
    type Error = Error;

    async fn send_to(
        &mut self,
        io: &mut (dyn AsyncWrite + Send + Unpin),
    ) -> std::result::Result<(), Self::Error> {
        self.0.send_to(io).await.map_err(Into::into)
    }
}
