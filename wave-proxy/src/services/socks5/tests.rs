#[allow(unused_imports)]
use super::{types::*, *};

#[test]
fn encode_decode() {
    let mut socks5 = Socks5Proxy::new();

    socks5.input(Input::Receive(Receive {
        proto: Protocol::Tcp,
        local: "127.0.0.1:77".parse().unwrap(),
        data: Bytes::from_static(HANDSHAKE_DATA),
        source: "127.0.0.1:88".parse().unwrap(),
    }));

    let res = socks5.poll_output().unwrap();
    assert_eq!(
        res,
        Output::Transmit(Transmit {
            proto: Protocol::Tcp,
            local: "127.0.0.1:77".parse().unwrap(),
            to: "127.0.0.1:88".parse().unwrap(),
            data: Bytes::from_static(HANDSHAKE_RESPONSE),
        })
    );

    socks5.input(Input::Receive(Receive {
        proto: Protocol::Tcp,
        local: "127.0.0.1:77".parse().unwrap(),
        data: Bytes::from_static(CONNECT_DATA),
        source: "127.0.0.1:88".parse().unwrap(),
    }));

    let res = socks5.poll_output().unwrap();
    if let Output::Connect(res) = res {
        let res = res.connected_success();
        assert_eq!(
            res,
            Transmit {
                proto: Protocol::Tcp,
                local: "127.0.0.1:77".parse().unwrap(),
                to: "127.0.0.1:88".parse().unwrap(),
                data: Bytes::from_static(CONNECT_RESPONSE),
            }
        )
    } else {
        panic!()
    };

    socks5.input(Input::Receive(Receive {
        proto: Protocol::Tcp,
        local: "127.0.0.1:77".parse().unwrap(),
        data: Bytes::from_static(REQ),
        source: "127.0.0.1:88".parse().unwrap(),
    }));

    let res = socks5.poll_output().unwrap();
    assert_eq!(
        res,
        Output::Transmit(Transmit {
            proto: Protocol::Tcp,
            local: "127.0.0.1:77".parse().unwrap(),
            to: "te.st:80".parse().unwrap(),
            data: Bytes::from_static(REQ),
        })
    )
}

const HANDSHAKE_DATA: &[u8] = &[0x5, 0x1, 0x00];

const HANDSHAKE_RESPONSE: &[u8] = &[0x5, 0x0];

const CONNECT_DATA: &[u8] = &[
    0x05, 0x01, 0x00, 0x03, 0x05, b't', b'e', b'.', b's', b't', 0x00, 0x50,
];

const CONNECT_RESPONSE: &[u8] = &[0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 77];

const REQ: &[u8] = b"GET / HTTP/1.1\r\n";

// const RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\n\r\n";
