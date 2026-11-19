use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    // ========
    /// Domain errors
    /// User related errors/ business rules
    #[error("User account is locked or disabled")]
    AccountLocked,

    #[error("User already exists")]
    AlreadyExists,

    // ========
    /// Cache errors
    /// Blacklist cache generic errors
    #[error("cache failure: {0}")]
    CacheError(Box<dyn std::error::Error + Send + Sync>),

    // ========
    /// Type casting errors
    /// Errors while converting between types
    /// Errors with regex
    #[error("decode/encode error: {0}")]
    CodecError(String),

    // ========
    /// Config errors
    /// Errors while reading env var
    #[error("Config error {0}")]
    Config(String),

    // ========
    /// Database errors
    /// Database generic errors
    #[error("Database or storage failure: {0}")]
    DatabaseError(Box<dyn std::error::Error + Send + Sync>),

    // ========
    /// Input errors
    /// Input validation errors
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
