pub mod socks5;

#[derive(Debug, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}
