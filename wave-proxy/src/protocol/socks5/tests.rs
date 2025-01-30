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
    let mut socks5 = Socks5::new("127.0.0.1:77".parse().unwrap());

    socks5.handle_input(
        Protocol::Tcp,
        "127.0.0.1:88".parse().unwrap(),
        &mut BytesMut::from(HANDSHAKE_DATA),
    );

    let res = socks5.poll_transmit().unwrap();
    assert_eq!(
        res,
        Transmit {
            proto: Protocol::Tcp,
            local: "127.0.0.1:77".parse().unwrap(),
            to: "127.0.0.1:88".parse().unwrap(),
            data: Bytes::from_static(HANDSHAKE_RESPONSE),
        }
    );

    socks5.handle_input(
        Protocol::Tcp,
        "127.0.0.1:88".parse().unwrap(),
        &mut BytesMut::from(CONNECT_DATA),
    );
    let event = socks5.poll_event().unwrap();
    assert_eq!(
        event,
        Event::ConnectToTarget {
            target: "te.st:80".parse().unwrap(),
        }
    );
    socks5.connect_with_status(ConnectedStatus::Succeeded);
    let res = socks5.poll_transmit().unwrap();

    assert_eq!(
        res,
        Transmit {
            proto: Protocol::Tcp,
            local: "127.0.0.1:77".parse().unwrap(),
            to: "127.0.0.1:88".parse().unwrap(),
            data: Bytes::from_static(CONNECT_RESPONSE),
        }
    );

    socks5.handle_input(
        Protocol::Tcp,
        "127.0.0.1:88".parse().unwrap(),
        &mut BytesMut::from(REQ),
    );

    let res = socks5.poll_transmit().unwrap();
    assert_eq!(
        res,
        Transmit {
            proto: Protocol::Tcp,
            local: "127.0.0.1:77".parse().unwrap(),
            to: "te.st:80".parse().unwrap(),
            data: Bytes::from_static(REQ),
        }
    );

    socks5.handle_input(
        Protocol::Tcp,
        "te.st:80".parse().unwrap(),
        &mut BytesMut::from(RESPONSE),
    );

    let res = socks5.poll_transmit().unwrap();
    assert_eq!(
        res,
        Transmit {
            proto: Protocol::Tcp,
            local: "127.0.0.1:77".parse().unwrap(),
            to: "127.0.0.1:88".parse().unwrap(),
            data: Bytes::from_static(RESPONSE),
        }
    );
}
