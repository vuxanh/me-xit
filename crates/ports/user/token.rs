use mockall::automock;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use error::UserResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claim {
    pub username: String,
    pub jti: String,
    pub exp: i64,
}

#[automock]
pub trait TokenPort: Send + Sync + Debug {
    fn generate_token(&self, username: &str, jti: &str, exp: i64) -> UserResult<String>;
    fn decode_token(&self, token: &str) -> UserResult<Claim>;
}
