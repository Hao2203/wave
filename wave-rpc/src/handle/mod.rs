use std::pin::Pin;

use futures::future::BoxFuture;
use tokio::io::{AsyncRead, AsyncWrite};

pub type RefReader<'a> = Pin<&'a mut (dyn AsyncRead + Send + 'a)>;

pub type RefWriter<'a> = Pin<&'a mut (dyn AsyncWrite + Send + 'a)>;

pub trait Handle<'a, Conn>
where
    Conn: AsyncRead + AsyncWrite,
{
    fn handle<'conn>(&'a self, conn: &'conn mut Conn) -> BoxFuture<'conn, anyhow::Result<()>>
    where
        'a: 'conn;
}
