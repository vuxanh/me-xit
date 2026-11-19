use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

use error::{UserError, UserResult};
use ports::{Claim, TokenPort};

#[derive(Debug)]
pub struct JsonWebTokenizer {
    secret: Vec<u8>,
}

impl TokenPort for JsonWebTokenizer {
    fn generate_token(&self, username: &str, jti: &str, ttl: i64) -> UserResult<String> {
        let exp = Utc::now().timestamp() + ttl;

        let claim = Claim {
            username: username.to_string(),
            jti: jti.to_string(),
            exp,
        };

        let token = encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(&self.secret),
        )
        .map_err(map_jwt_err)?;

        Ok(token)
    }

    fn decode_token(&self, token: &str) -> UserResult<Claim> {
        let claim = decode::<Claim>(
            token,
            &DecodingKey::from_secret(&self.secret),
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        )
        .map_err(map_jwt_err)?
        .claims;

        Ok(claim)
    }
}

impl JsonWebTokenizer {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            secret: secret.to_owned(),
        }
    }
}

fn map_jwt_err(e: jsonwebtoken::errors::Error) -> UserError {
    let message = e.to_string();

    match e.into_kind() {
        jsonwebtoken::errors::ErrorKind::InvalidToken => UserError::InvalidToken(message),
        _ => UserError::CodecError(message),
    }
}
