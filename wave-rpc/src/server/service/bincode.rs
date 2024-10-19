use super::{FromRequest, Request, Response, Result, ToResponse};
use crate::Body;

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

impl<T> ToResponse for T
where
    T: serde::Serialize,
{
    fn to_response(&self) -> Result<Response> {
        let bytes = bincode::serialize(&self)?;
        let body = Body::new(bytes.into());
        let response = Response::success(body);
        Ok(response)
    }
}
