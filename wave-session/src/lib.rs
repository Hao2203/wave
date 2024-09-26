pub mod error;
pub mod message;
pub mod service;
pub mod session;
pub mod user;

#[cfg(test)]
mod test;

pub trait SessionHandle {
    fn name(&self) -> &str;
}
