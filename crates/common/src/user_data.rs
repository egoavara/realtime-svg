
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use serde::{Deserialize, Serialize};

use crate::errors::ApiError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserData {
    pub password_argon2: String,
}

impl UserData {
    pub fn create(
        argon: &Argon2<'_>,
        salt: &SaltString,
        password_argon2: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let password_hash = argon
            .hash_password(password_argon2.into().as_bytes(), salt)?
            .to_string();
        Ok(Self {
            password_argon2: password_hash,
        })
    }
}
