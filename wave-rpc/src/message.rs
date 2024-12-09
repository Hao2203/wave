use crate::{body::MessageBody, error::Error};
use std::future::Future;

pub mod stream;

pub trait FromBody<Ctx> {
    type Error: Into<Error>;

    fn from_body(
        ctx: &mut Ctx,
        body: impl MessageBody,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;
}

pub trait IntoBody {
    fn into_body(self) -> impl MessageBody;
}
