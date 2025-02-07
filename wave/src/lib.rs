use derive_more::{Display, Error, From};
use iroh::endpoint::{RecvStream, SendStream};
use std::pin::pin;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};

pub mod cli;
pub mod client;
pub mod server;
#[cfg(test)]
mod tests;

pub const ALPN: &[u8] = b"wave";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WavePacket {
    pub port: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub iroh::PublicKey);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let encoder = data_encoding::BASE32_DNSSEC;
        let bs32 = encoder.encode_display(self.0.as_bytes());
        write!(f, "{}", bs32)
    }
}

#[derive(Debug, Display, From, Error)]
pub enum NodeIdParsingError {
    Decode(data_encoding::DecodeError),
    Key(ed25519_dalek::SignatureError),
}

impl std::str::FromStr for NodeId {
    type Err = NodeIdParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = data_encoding::BASE32_DNSSEC.decode(s.as_bytes())?;
        let public_key = iroh::PublicKey::try_from(bytes.as_slice())?;
        Ok(NodeId(public_key))
    }
}

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
