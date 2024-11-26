use std::future::Future;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait SendTo<T> {
    fn send(
        &self,
        writer: &mut (dyn AsyncWrite + Sync + Unpin),
        item: T,
    ) -> impl Future<Output = Result<(), std::io::Error>> + Send;
}

pub trait Recv<T> {
    fn recv(
        &self,
        reader: &mut (dyn AsyncRead + Sync + Unpin),
    ) -> impl Future<Output = Result<T, std::io::Error>> + Send;
}
