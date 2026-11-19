use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::Error::PasswordInvalid};

use error::{UserError, UserResult};
use ports::HasherPort;

#[derive(Debug)]
pub struct ArgonHasher<'a> {
    client: Argon2<'a>,
}

impl<'a> ArgonHasher<'a> {
    pub fn new() -> Self {
        Self {
            client: Argon2::default(),
        }
    }
}

impl<'a> HasherPort for ArgonHasher<'a> {
    fn hash(&self, password: &str) -> UserResult<String> {
        let password = self
            .client
            .hash_password(password.as_bytes())
            .map_err(map_hasher_err)?;

        Ok(password.to_string())
    }

    fn verify(&self, password: &str, password_hash: &str) -> UserResult<bool> {
        let result = self
            .client
            .verify_password(password.as_bytes(), password_hash);

        match result {
            Ok(_) => Ok(true),
            Err(PasswordInvalid) => Ok(false),
            Err(e) => Err(map_hasher_err(e)),
        }
    }
}

fn map_hasher_err(e: argon2::password_hash::Error) -> UserError {
    UserError::CodecError(e.to_string())
}
