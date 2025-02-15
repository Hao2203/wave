use iroh::endpoint::{RecvStream, SendStream};
use std::pin::pin;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};

pub mod cli;
pub mod client;
pub mod config;
pub mod server;
pub mod store;
#[cfg(test)]
mod tests;

pub const ALPN: &[u8] = b"wave";

pub enum Stream {
    Iroh(SendStream, RecvStream),
    Tcp(TcpStream),
}

impl AsyncRead for Stream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Stream::Iroh(_, recv_stream) => pin!(recv_stream).poll_read(cx, buf),
            Stream::Tcp(stream) => pin!(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            Stream::Iroh(send_stream, _) => pin!(send_stream).poll_write(cx, buf),
            Stream::Tcp(stream) => pin!(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Stream::Iroh(send_stream, _) => pin!(send_stream).poll_flush(cx),
            Stream::Tcp(stream) => pin!(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Stream::Iroh(send_stream, _) => pin!(send_stream).poll_shutdown(cx),
            Stream::Tcp(stream) => pin!(stream).poll_shutdown(cx),
        }
    }
}
