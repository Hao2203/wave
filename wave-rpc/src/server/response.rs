use crate::Body;
use crate::Response;
use crate::Result;

pub trait ToResponse {
    fn to_response(&self) -> Result<Response>;
}

#[cfg(feature = "bincode")]
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
