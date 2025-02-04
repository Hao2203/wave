use bytes::BytesMut;

#[allow(unused_imports)]
use super::{types::*, *};

const HANDSHAKE_DATA: &[u8] = &[0x5, 0x1, 0x00];

const HANDSHAKE_RESPONSE: &[u8] = &[0x5, 0x0];

const CONNECT_DATA: &[u8] = &[
    0x05, 0x01, 0x00, 0x03, 0x05, b't', b'e', b'.', b's', b't', 0x00, 0x50,
];

const CONNECT_RESPONSE: &[u8] = &[0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 77];

const REQ: &[u8] = b"GET / HTTP/1.1\r\n";

const RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\n\r\n";

#[test]
fn test1() {
    let socks5 = NoAuthHandshake::new(
        "127.0.0.1:77".parse().unwrap(),
        "127.0.0.1:88".parse().unwrap(),
    );

    let req = codec::decode_handshake_request(&mut BytesMut::from(HANDSHAKE_DATA))
        .unwrap()
        .unwrap();

    let (transmit, socks5) = socks5.handshake(req);

    assert_eq!(transmit, Transmit {
        proto: Protocol::Tcp,
        local: "127.0.0.1:77".parse().unwrap(),
        to: "127.0.0.1:88".parse().unwrap(),
        data: Bytes::from_static(HANDSHAKE_RESPONSE),
    });

    let request = codec::decode_connect_request(&mut BytesMut::from(CONNECT_DATA))
        .unwrap()
        .unwrap();
    let status = ConnectedStatus::Succeeded;
    let (transmit, socks5) = socks5.unwrap().connect(request, status);
    assert_eq!(transmit, Transmit {
        proto: Protocol::Tcp,
        local: "127.0.0.1:77".parse().unwrap(),
        to: "127.0.0.1:88".parse().unwrap(),
        data: Bytes::from_static(CONNECT_RESPONSE),
    });

    let mut socks5 = socks5.unwrap();
    let res = socks5.relay("127.0.0.1:88".parse().unwrap(), Bytes::from_static(REQ));

    assert_eq!(res, Transmit {
        proto: Protocol::Tcp,
        local: "127.0.0.1:77".parse().unwrap(),
        to: "te.st:80".parse().unwrap(),
        data: Bytes::from_static(REQ),
    });

    let res = socks5.relay("te.st:80".parse().unwrap(), Bytes::from_static(RESPONSE));
    assert_eq!(res, Transmit {
        proto: Protocol::Tcp,
        local: "127.0.0.1:77".parse().unwrap(),
        to: "127.0.0.1:88".parse().unwrap(),
        data: Bytes::from_static(RESPONSE),
    });
}
