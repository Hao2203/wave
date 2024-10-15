// use crate::transport::{error::Result, FromRequest};
// use serde::de::DeserializeOwned;

// pub struct Rmp<T>(pub T);

// impl<T> FromRequest for Rmp<T>
// where
//     T: DeserializeOwned + Send,
// {
//     async fn from_request(req: crate::Request<'_>) -> Result<Self> {
//         todo!()
//     }
// }
