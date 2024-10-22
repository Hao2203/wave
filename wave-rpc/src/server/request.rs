use crate::Body;
use crate::{Request, Result};
use std::future::Future;

pub trait FromRequest: Sized {
    fn from_request(req: &mut Request) -> impl Future<Output = Result<Self>> + Send;
}

#[cfg(feature = "bincode")]
impl<T> FromRequest for T
where
    T: serde::de::DeserializeOwned,
{
    async fn from_request(req: &mut Request) -> Result<Self> {
        let bytes = req.body().as_slice();
        let value = bincode::deserialize(bytes)?;
        Ok(value)
    }
}
