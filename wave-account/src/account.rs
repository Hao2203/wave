#![allow(unused)]
use derive_more::From;
use ed25519_dalek::{SigningKey, VerifyingKey};

pub struct Account {
    name: String,
}

#[derive(Debug, Clone, From)]
#[from(forward)]
pub struct AccountId(VerifyingKey);

impl AccountId {}

pub struct AccountSecretKey(SigningKey);

impl AccountSecretKey {
    pub fn new() -> Self {
        let mut rng = rand::rngs::OsRng;
        AccountSecretKey(SigningKey::generate(&mut rng))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn account_id(&self) -> AccountId {
        AccountId(VerifyingKey::from(&self.0))
    }
}

impl Default for AccountSecretKey {
    fn default() -> Self {
        Self::new()
    }
}
