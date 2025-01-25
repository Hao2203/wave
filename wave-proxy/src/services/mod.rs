use derive_more::derive::Display;

pub mod socks5;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    #[display("TCP")]
    Tcp,
    #[display("UDP")]
    Udp,
}
