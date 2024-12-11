use crate::error::RpcError;
use serde::Deserialize;

use super::*;
pub struct Bincode<T>(pub T);

impl<T, Ctx> FromBody<Ctx> for Bincode<T>
where
    T: for<'a> Deserialize<'a>,
    Ctx: Send,
{
    type Error = Error;
    async fn from_body(
        ctx: &mut Ctx,
        body: impl crate::body::MessageBody,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let bytes = Bytes::from_body(ctx, body).await?;
        let data = ::bincode::deserialize(&bytes)?;
        Ok(Bincode(data))
    }
}

impl RpcError for ::bincode::Error {
    fn code(&self) -> crate::code::Code {
        todo!()
    }
}
