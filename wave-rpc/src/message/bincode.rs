use derive_more::derive::{Display, From};
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
        body: impl Stream<Item = Result<Bytes, io::Error>> + Unpin + Send + 'static,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let bytes = Bytes::from_body(ctx, body).await?;
        let data = ::bincode::deserialize(&bytes)?;
        Ok(Bincode(data))
    }
}

#[derive(Debug, Display, From, derive_more::Error)]
pub enum Error {
    Io(io::Error),
    Bincode(::bincode::Error),
}
