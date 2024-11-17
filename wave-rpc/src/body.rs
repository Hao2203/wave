use async_trait::async_trait;

use crate::{error::Error, message::WriteIn};

pub struct Body<'a>(pub Box<dyn WriteIn<Error = Error> + Send + 'a>);

impl<'a> Body<'a> {
    pub fn new(body: impl WriteIn<Error: Into<Error>> + Send + 'a) -> Self {
        Self(Box::new(BodyInner(body)))
    }
}

struct BodyInner<T>(T);

#[async_trait]
impl<T> WriteIn for BodyInner<T>
where
    T: WriteIn<Error: Into<Error>> + Send,
{
    type Error = Error;

    async fn write_in(
        &mut self,
        io: &mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> std::result::Result<(), Self::Error> {
        self.0.write_in(io).await.map_err(Into::into)
    }
}
