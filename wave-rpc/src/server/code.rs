#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[repr(u16)]
pub enum ErrorCode {
    #[error("service not found")]
    ServiceNotFound,
}
