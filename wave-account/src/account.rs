#![allow(unused)]
use derive_more::{Display, From};
use ed25519_dalek::{SigningKey, VerifyingKey};

#[derive(Debug, Clone, From)]
#[from(forward)]
pub struct Id(VerifyingKey);

impl Id {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl From<SecretKey> for Id {
    fn from(value: SecretKey) -> Self {
        value.account_id()
    }
}

#[derive(Debug, Clone, From)]
pub struct SecretKey(SigningKey);

impl SecretKey {
    pub fn new() -> Self {
        let mut rng = rand::rngs::OsRng;
        SecretKey(SigningKey::generate(&mut rng))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn account_id(&self) -> Id {
        Id(VerifyingKey::from(&self.0))
    }
}

impl Default for SecretKey {
    fn default() -> Self {
        Self::new()
    }
}
