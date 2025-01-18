// #![allow(unused_imports)]
use super::*;
use crate::{
    error::Context, Connection, Error, ErrorInner, Incoming, ProxyApp, ProxyService, ProxyStatus,
    Result, Target,
};
use bytes::{Buf, BufMut, Bytes, BytesMut};
pub use consts::*;
use fast_socks5::{
    parse_udp_request,
    server::{AcceptAuthentication, Config, Socks5Socket},
    util::target_addr::TargetAddr,
    ReplyError, Socks5Command, SocksError,
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    net::UdpSocket,
    try_join,
};

#[rustfmt::skip]
pub mod consts {
    pub const SOCKS5_VERSION:                          u8 = 0x05;

    pub const SOCKS5_AUTH_METHOD_NONE:                 u8 = 0x00;
    pub const SOCKS5_AUTH_METHOD_GSSAPI:               u8 = 0x01;
    pub const SOCKS5_AUTH_METHOD_PASSWORD:             u8 = 0x02;
    pub const SOCKS5_AUTH_METHOD_NOT_ACCEPTABLE:       u8 = 0xff;

    pub const SOCKS5_CMD_TCP_CONNECT:                  u8 = 0x01;
    pub const SOCKS5_CMD_TCP_BIND:                     u8 = 0x02;
    pub const SOCKS5_CMD_UDP_ASSOCIATE:                u8 = 0x03;

    pub const SOCKS5_ADDR_TYPE_IPV4:                   u8 = 0x01;
    pub const SOCKS5_ADDR_TYPE_DOMAIN_NAME:            u8 = 0x03;
    pub const SOCKS5_ADDR_TYPE_IPV6:                   u8 = 0x04;

    pub const SOCKS5_REPLY_SUCCEEDED:                  u8 = 0x00;
    pub const SOCKS5_REPLY_GENERAL_FAILURE:            u8 = 0x01;
    pub const SOCKS5_REPLY_CONNECTION_NOT_ALLOWED:     u8 = 0x02;
    pub const SOCKS5_REPLY_NETWORK_UNREACHABLE:        u8 = 0x03;
    pub const SOCKS5_REPLY_HOST_UNREACHABLE:           u8 = 0x04;
    pub const SOCKS5_REPLY_CONNECTION_REFUSED:         u8 = 0x05;
    pub const SOCKS5_REPLY_TTL_EXPIRED:                u8 = 0x06;
    pub const SOCKS5_REPLY_COMMAND_NOT_SUPPORTED:      u8 = 0x07;
    pub const SOCKS5_REPLY_ADDRESS_TYPE_NOT_SUPPORTED: u8 = 0x08;
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Method {
    None = SOCKS5_AUTH_METHOD_NONE,
    Gssapi = SOCKS5_AUTH_METHOD_GSSAPI,
    Password = SOCKS5_AUTH_METHOD_PASSWORD,
    NotAcceptable = SOCKS5_AUTH_METHOD_NOT_ACCEPTABLE,
}

impl TryFrom<u8> for Method {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            SOCKS5_AUTH_METHOD_NONE => Ok(Method::None),
            SOCKS5_AUTH_METHOD_GSSAPI => Ok(Method::Gssapi),
            SOCKS5_AUTH_METHOD_PASSWORD => Ok(Method::Password),
            SOCKS5_AUTH_METHOD_NOT_ACCEPTABLE => Ok(Method::NotAcceptable),
            _ => Err(Error::new(
                ErrorInner::UnSupportedProxyProtocol,
                "Invalid socks5 method",
            )),
        }
    }
}

pub struct Socks5Proxy {
    source: SocketAddr,
    local: SocketAddr,
}

impl Socks5Proxy {
    pub fn process_consult_request(&self, request: ConsultRequest) -> Result<Bytes> {
        let method = request.methods[0];
        match method {
            Method::None => Ok(ConsultResponse(method).into()),
            _ => Err(Error::new(
                ErrorInner::UnSupportedProxyProtocol,
                "Invalid socks5 request",
            )),
        }
    }
}

/// +----+----------+----------+
/// |VER | NMETHODS | METHODS  |
/// +----+----------+----------+
/// | 1  |    1     | 1 to 255 |
/// +----+----------+----------+
pub struct ConsultRequest {
    pub n_methods: u8,
    pub methods: Vec<Method>,
}

impl ConsultRequest {
    pub fn new(version: u8, n_methods: u8, methods: Bytes) -> Result<Self> {
        if version != 5 {
            return Err(Error::new(
                ErrorInner::UnSupportedProxyProtocol,
                "Invalid socks5 version",
            ));
        }
        let methods = methods
            .into_iter()
            .map(|x| x.try_into())
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { n_methods, methods })
    }
}

impl TryFrom<&mut bytes::Bytes> for ConsultRequest {
    type Error = Error;
    fn try_from(buf: &mut bytes::Bytes) -> Result<Self> {
        if buf.len() < 2 {
            return Err(Error::new(
                ErrorInner::UnSupportedProxyProtocol,
                "Invalid socks5 request",
            ));
        }
        let version = buf.get_u8();
        let n_methods = buf.get_u8();
        let methods = buf.split_to(n_methods as usize);

        ConsultRequest::new(version, n_methods, methods)
    }
}

/// +----+--------+
/// |VER | METHOD |
/// +----+--------+
/// | 1  |   1    |
/// +----+--------+
pub struct ConsultResponse(pub Method);

impl From<ConsultResponse> for Bytes {
    fn from(value: ConsultResponse) -> Self {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_u8(5);
        buf.put_u8(value.0 as u8);
        buf.freeze()
    }
}

pub enum State {
    Pending,
    Consult(Transmit),
}

pub struct Socks5 {
    timeout: Duration,
}

impl Default for Socks5 {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
        }
    }
}

impl Socks5 {
    async fn tcp_transfer<T: Connection + Unpin>(
        &self,
        target: &Target,
        mut inbound: T,
    ) -> Result<()> {
        // TCP connect with timeout, to avoid memory leak for connection that takes forever
        let mut outbound = match target {
            Target::Domain(url, port) => {
                tokio::time::timeout(
                    self.timeout,
                    tokio::net::TcpStream::connect((url.as_ref(), *port)),
                )
                .await
            }
            Target::Ip(addr) => {
                tokio::time::timeout(self.timeout, tokio::net::TcpStream::connect(addr)).await
            }
        }
        .context("Connect timeout when connecting to tcp remote")?
        .context("Can't connect to remote destination")?;

        inbound
            .write(&new_reply(
                &ReplyError::Succeeded,
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
            ))
            .await
            .context("Can't write successful reply")?;

        inbound.flush().await.context("Can't flush the reply!")?;

        self.copy_bidirectional(&mut outbound, &mut inbound).await?;

        Ok(())
    }

    async fn copy_bidirectional<A, B>(&self, mut a: A, mut b: B) -> Result<()>
    where
        A: AsyncRead + AsyncWrite + Unpin,
        B: AsyncRead + AsyncWrite + Unpin,
    {
        tokio::io::copy_bidirectional(&mut a, &mut b)
            .await
            .context("Can't copy the connection!")?;
        Ok(())
    }

    async fn execute_command_udp_assoc<T: Connection + Unpin>(
        &self,
        mut inbound: T,
        local_addr: SocketAddr,
    ) -> Result<()> {
        // Listen with UDP6 socket, so the client can connect to it with either
        // IPv4 or IPv6.
        let peer_sock = UdpSocket::bind("[::]:0")
            .await
            .context("Can't bind UDP socket")?;

        // Respect the pre-populated reply IP address.
        inbound
            .write(&new_reply(&ReplyError::Succeeded, local_addr))
            .await
            .context("Can't write successful reply")?;

        self.transfer_udp(peer_sock).await?;

        Ok(())
    }

    async fn handle_udp_request(&self, inbound: &UdpSocket, outbound: &UdpSocket) -> Result<()> {
        let mut buf = vec![0u8; 0x10000];
        loop {
            let (size, client_addr) = inbound
                .recv_from(&mut buf)
                .await
                .context("Can't recv udp packet while handling udp request")?;

            inbound
                .connect(client_addr)
                .await
                .context("Can't connect to client address while handling udp request")?;

            let (frag, target_addr, data) = parse_udp_request(&buf[..size])
                .await
                .context("Can't parse udp request")?;

            if frag != 0 {
                // debug!("Discard UDP frag packets sliently.");
                return Ok(());
            }

            // debug!("Server forward to packet to {}", target_addr);
            let mut target_addr = target_addr
                .resolve_dns()
                .await
                .context("Can't resolve dns")?
                .to_socket_addrs()
                .context("Can't convert to socket address")?
                .next()
                .context("Can't get next socket address")?;

            target_addr.set_ip(match target_addr.ip() {
                std::net::IpAddr::V4(v4) => std::net::IpAddr::V6(v4.to_ipv6_mapped()),
                v6 @ std::net::IpAddr::V6(_) => v6,
            });

            outbound
                .send_to(data, target_addr)
                .await
                .context("Can't send packet")?;
        }
    }

    async fn handle_udp_response(&self, inbound: &UdpSocket, outbound: &UdpSocket) -> Result<()> {
        let mut buf = vec![0u8; 0x10000];
        loop {
            let (size, remote_addr) = outbound
                .recv_from(&mut buf)
                .await
                .context("Can't recv udp packet while handling udp response")?;
            // debug!("Recieve packet from {}", remote_addr);

            let mut data =
                fast_socks5::new_udp_header(remote_addr).context("Can't create udp header")?;
            data.extend_from_slice(&buf[..size]);

            inbound
                .send(&data)
                .await
                .context("Can't send packet while handling udp response")?;
        }
    }

    async fn transfer_udp(&self, inbound: UdpSocket) -> Result<()> {
        let outbound = UdpSocket::bind("[::]:0")
            .await
            .context("Can't bind UDP socket")?;

        let req_fut = self.handle_udp_request(&inbound, &outbound);
        let res_fut = self.handle_udp_response(&inbound, &outbound);
        match try_join!(req_fut, res_fut) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl<T: ProxyApp + Sync + 'static> ProxyService<T> for Socks5 {
    async fn serve<'a>(&self, app: &T, incoming: Incoming<'a>) -> Result<ProxyStatus<'a>> {
        let local_addr = incoming.local_addr;
        let mut ctx = app.new_ctx();

        let mut config = Config::<AcceptAuthentication>::default();
        config.set_execute_command(false);
        config.set_dns_resolve(false);
        let mut socks5 = Socks5Socket::new(incoming, Arc::new(config))
            .upgrade_to_socks5()
            .await
            .context("failed to upgrade to socks5")?;
        // println!("socks5 upgrade success");

        let target: Target = socks5
            .target_addr()
            .ok_or(Error::new(ErrorInner::GetTargetFailed, "get target failed"))?
            .into();

        match socks5.cmd() {
            None => Ok(ProxyStatus::Continue(socks5.into_inner())),
            Some(cmd) => match cmd {
                Socks5Command::TCPConnect => {
                    let tunnel = app.upstream(&mut ctx, &target).await?;
                    let mut socks5 = socks5.into_inner();

                    if let Some(mut tunnel) = tunnel {
                        reply_success(&mut socks5, local_addr).await?;
                        self.copy_bidirectional(&mut tunnel, &mut socks5).await?;

                        app.after_forward(&mut ctx, tunnel).await?;
                    } else {
                        self.tcp_transfer(&target, &mut socks5).await?;
                    }

                    Ok(ProxyStatus::Success)
                }
                Socks5Command::UDPAssociate => {
                    self.execute_command_udp_assoc(&mut socks5, local_addr)
                        .await?;
                    Ok(ProxyStatus::Success)
                }

                _ => Err(Error::new(
                    ErrorInner::UnSupportedProxyProtocol,
                    "parse command failed",
                )),
            },
        }
    }
}

async fn reply_success(io: &mut (impl Connection + Unpin), addr: SocketAddr) -> Result<()> {
    let reply = new_reply(&ReplyError::Succeeded, addr);
    reply_to(io, reply).await
}

async fn reply_to(io: &mut (impl Connection + Unpin), reply: impl AsRef<[u8]>) -> Result<()> {
    io.write_all(reply.as_ref())
        .await
        .context("failed to write reply")
}

/// Generate reply code according to the RFC.
fn new_reply(error: &ReplyError, sock_addr: SocketAddr) -> Vec<u8> {
    let (addr_type, mut ip_oct, mut port) = match sock_addr {
        SocketAddr::V4(sock) => (
            consts::SOCKS5_ADDR_TYPE_IPV4,
            sock.ip().octets().to_vec(),
            sock.port().to_be_bytes().to_vec(),
        ),
        SocketAddr::V6(sock) => (
            consts::SOCKS5_ADDR_TYPE_IPV6,
            sock.ip().octets().to_vec(),
            sock.port().to_be_bytes().to_vec(),
        ),
    };

    let mut reply = vec![
        consts::SOCKS5_VERSION,
        error.as_u8(), // transform the error into byte code
        0x00,          // reserved
        addr_type,     // address type (ipv4, v6, domain)
    ];
    reply.append(&mut ip_oct);
    reply.append(&mut port);

    reply
}

impl From<&TargetAddr> for crate::Target {
    fn from(addr: &TargetAddr) -> Self {
        match addr {
            TargetAddr::Ip(ip) => Self::Ip(*ip),
            TargetAddr::Domain(domain, port) => Self::Domain(domain.clone().into(), *port),
        }
    }
}

impl From<TargetAddr> for crate::Target {
    fn from(addr: TargetAddr) -> Self {
        match addr {
            TargetAddr::Ip(ip) => Self::Ip(ip),
            TargetAddr::Domain(domain, port) => Self::Domain(domain.into(), port),
        }
    }
}

impl From<SocksError> for ErrorInner {
    fn from(value: SocksError) -> Self {
        type E = SocksError;
        match value {
            E::Io(e) => e.into(),
            E::InvalidHeader {
                expected: _,
                found: _,
            }
            | E::UnsupportedSocksVersion(_) => ErrorInner::UnSupportedProxyProtocol,
            _ => ErrorInner::Unexpected,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::server::ProxyServer;
    use fast_socks5::client::*;
    use std::io::Cursor;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test() {
        pub struct TestApp;
        impl ProxyApp for TestApp {
            type Ctx = ();
            type Upstream = Cursor<Vec<u8>>;
            fn new_ctx(&self) -> Self::Ctx {}
            async fn upstream(
                &self,
                _ctx: &mut Self::Ctx,
                target: &Target,
            ) -> Result<Option<Self::Upstream>> {
                assert_eq!(*target, Target::Domain("www.example.com".into(), 80));
                Ok(Some(Cursor::new(vec![])))
            }
            async fn after_forward(
                &self,
                _ctx: &mut Self::Ctx,
                tunnel: Self::Upstream,
            ) -> Result<()> {
                assert_eq!(tunnel.into_inner(), http_header_data());
                Ok(())
            }
        }

        let server_task = tokio::spawn(async move {
            let listener = TcpListener::bind("127.0.0.1:1234").await.unwrap();
            let (stream, addr) = listener.accept().await.unwrap();
            let server = ProxyServer::builder(TestApp)
                .add_proxy(Socks5::default())
                .build()
                .unwrap();
            server.serve(stream, addr).await.unwrap();
        });

        let client_task = tokio::spawn(async move {
            let mut client = Socks5Stream::connect_raw(
                Socks5Command::TCPConnect,
                "127.0.0.1:1234",
                "www.example.com".into(),
                80,
                None,
                Default::default(),
            )
            .await
            .unwrap();
            client.write_all(&http_header_data()).await.unwrap();
        });

        server_task.await.unwrap();
        client_task.await.unwrap();
    }

    fn http_header_data() -> Vec<u8> {
        let domain = "www.example.com".to_string();
        // construct our request, with a dynamic domain
        let mut headers = vec![];
        headers.extend_from_slice("GET / HTTP/1.1\r\nHost: ".as_bytes());
        headers.extend_from_slice(domain.as_bytes());
        headers.extend_from_slice(
            "\r\nUser-Agent: fast-socks5/0.1.0\r\nAccept: */*\r\n\r\n".as_bytes(),
        );

        headers
    }

    async fn spawn_test_server() {
        let route = axum::Router::new().route("/", axum::routing::get("Hello, World!"));
        let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
        tokio::spawn(async move {
            axum::serve(listener, route).await.unwrap();
        });
    }

    #[tokio::test]
    async fn test_http() {
        pub struct TestApp;
        impl ProxyApp for TestApp {
            type Ctx = ();
            type Upstream = Cursor<Vec<u8>>;
            fn new_ctx(&self) -> Self::Ctx {}
            async fn upstream(
                &self,
                _ctx: &mut Self::Ctx,
                _target: &Target,
            ) -> Result<Option<Self::Upstream>> {
                // println!("target: {:?}", _target);
                Ok(None)
            }
        }

        spawn_test_server().await;
        let server_task = tokio::spawn(async move {
            let listener = TcpListener::bind("127.0.0.1:1234").await.unwrap();
            let (stream, addr) = listener.accept().await.unwrap();
            let server = ProxyServer::builder(TestApp)
                .add_proxy(Socks5::default())
                .build()
                .unwrap();
            server.serve(stream, addr).await.unwrap();
        });
        let client_task = tokio::spawn(async move {
            let proxy = reqwest::Proxy::all("socks5://127.0.0.1:1234").unwrap();
            let client = reqwest::Client::builder().proxy(proxy).build().unwrap();
            let res = client.get("http://127.0.0.1:8000/").send().await.unwrap();
            assert_eq!(res.text().await.unwrap(), "Hello, World!");
        });

        server_task.await.unwrap();
        client_task.await.unwrap();
    }
}
