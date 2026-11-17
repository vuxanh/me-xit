use async_trait::async_trait;
use error::UserResult;

pub struct LoginCommand {
    pub username: String,
    pub password: String,
}

pub struct LogoutCommand {
    pub token: String,
}

pub struct ValidateQuery {
    pub token: String,
}

pub struct RegisterCommand {
    pub username: String,
    pub password: String,
}

#[async_trait]
pub trait Authenticator: Send + Sync {
    async fn handle_login(&self, command: &LoginCommand) -> UserResult<LoginResponse>;
    async fn handle_validate(&self, query: &ValidateQuery) -> UserResult<bool>;
    async fn handle_logout(&self, command: &LogoutCommand) -> UserResult<()>;
    async fn handle_register(&self, command: &RegisterCommand) -> UserResult<()>;
}

pub struct LoginResponse {
    pub token: String,
    pub ttl: i64,
}
