use iroh::{
    endpoint::{RecvStream, SendStream},
    Endpoint,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    net::SocketAddr,
    pin::pin,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};
use wave_proxy::{
    protocol::socks5::{codec, NoAuthHandshake, Relay, Transmit},
    Address,
};

pub struct Client {
    local: SocketAddr,
    listener: tokio::net::TcpListener,
    endpoint: Endpoint,
}

impl Client {
    pub async fn run(self) -> anyhow::Result<()> {
        loop {
            let (stream, local) = self.listener.accept().await?;
            let handler = Handler {
                local,
                endpoint: self.endpoint.clone(),
            };
            tokio::spawn(async move {
                handler.handle(stream).await.unwrap();
            });
        }
    }
}

struct Handler {
    local: SocketAddr,
    endpoint: Endpoint,
}

impl Handler {
    async fn handle(self, mut stream: tokio::net::TcpStream) -> anyhow::Result<()> {
        let remote = stream.peer_addr().unwrap();
        let socks5 = NoAuthHandshake::new(self.local, remote);
        let mut buf = Vec::with_capacity(100);
        stream.read_buf(&mut buf).await.unwrap();
        let req = codec::decode_consult_request(&mut buf.as_slice()).unwrap();
        let (transmit, socks5) = socks5.handshake(req);
        todo!()
    }

    async fn handle_transmit(&self, transmit: Transmit) -> anyhow::Result<()> {
        todo!()
    }
}

pub struct Transport {
    local: SocketAddr,
    endpoint: Endpoint,
    streams: HashMap<Address, Stream>,
}

pub enum Stream {
    Iroh(SendStream, RecvStream),
    Tcp(TcpStream),
}

impl Transport {
    async fn handle(&mut self, Transmit { to, mut data, .. }: Transmit) -> anyhow::Result<()> {
        let stream = match self.streams.entry(to) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => match entry.key() {
                Address::Ip(ip) => {
                    let stream = TcpStream::connect(ip).await?;
                    entry.insert(Stream::Tcp(stream))
                }
                Address::Domain(domain, port) => {
                    todo!()
                }
            },
        };
        stream.write_all_buf(&mut data).await?;

        Ok(())
    }
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
