use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User account is locked or disabled")]
    AccountLocked,

    #[error("User already exists")]
    AlreadyExists,

    #[error("cache failure: {0}")]
    CacheError(Box<dyn std::error::Error + Send + Sync>),

    #[error("decode/encode error: {0}")]
    CodecError(String),

    #[error("Config error {0}")]
    Config(String),

    #[error("Database or storage failure: {0}")]
    DatabaseError(Box<dyn std::error::Error + Send + Sync>),

    #[error("Invalid credentials provided: {0}")]
    InvalidCredentials(String),

    #[error("Authentication token is invalid or malformed: {0}")]
    InvalidToken(String),

    #[error("Missing authorization header or token")]
    MissingToken,

    #[error("Authentication token has expired")]
    TokenExpired,
}

pub type UserResult<T> = std::result::Result<T, UserError>;
