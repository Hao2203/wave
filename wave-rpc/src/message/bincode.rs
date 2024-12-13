use derive_more::derive::{Display, From};
use serde::{Deserialize, Serialize};

use super::*;
pub struct Bincode<T>(pub T);

impl<T> FromStream for Bincode<T>
where
    T: for<'a> Deserialize<'a>,
{
    type Error = Error;
    async fn from_stream(
        body: impl Stream<Item = Bytes> + Unpin + Send + 'static,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let bytes = Bytes::from_stream(body).await;
        match bytes {
            Ok(bytes) => {
                let data = ::bincode::deserialize(&bytes)?;
                Ok(Bincode(data))
            }
        }
    }
}

impl<T> IntoStream for Bincode<T>
where
    T: Serialize,
{
    type Error = Error;
    fn into_stream(self) -> impl TryBytesStream<Error = Self::Error> {
        let bytes = ::bincode::serialize(&self.0)
            .map(Bytes::from)
            .map_err(Error::Bincode);
        futures_lite::stream::once(bytes)
    }
}

#[derive(Debug, Display, From, derive_more::Error)]
pub enum Error {
    Io(io::Error),
    Bincode(::bincode::Error),
}
