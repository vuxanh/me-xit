use async_trait::async_trait;
use error::UserResult;

use crate::Claim;

pub struct LoginCommand {
    pub username: String,
    pub password: String,
}

pub struct LogoutCommand {
    pub claim: Claim,
}

pub struct ValidateQuery {
    pub claim: Claim,
}

pub struct RegisterCommand {
    pub username: String,
    pub password: String,
}

#[async_trait]
pub trait Authenticator: Send + Sync {
    async fn handle_login(&self, command: &LoginCommand) -> UserResult<LoginResponse>;
    async fn handle_validate(&self, query: &ValidateQuery) -> UserResult<ValidateResponse>;
    async fn handle_logout(&self, command: &LogoutCommand) -> UserResult<()>;
    async fn handle_register(&self, command: &RegisterCommand) -> UserResult<()>;
}

#[derive(Debug)]
pub struct LoginResponse {
    pub token: String,
    pub ttl: i64,
}

#[derive(Debug)]
pub struct ValidateResponse {
    pub username: String,
    pub role: String,
}
