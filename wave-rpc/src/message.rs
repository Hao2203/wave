use crate::body::MessageBody;
use std::future::Future;

pub mod stream;

pub trait FromBody {
    type Error: core::error::Error + Send;

    fn from_body(body: impl MessageBody) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;
}

pub trait IntoBody {
    fn into_body(self) -> impl MessageBody;
}
