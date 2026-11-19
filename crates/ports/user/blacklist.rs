use async_trait::async_trait;

use error::UserResult;
use mockall::automock;
use std::fmt::Debug;

#[automock]
#[async_trait]
pub trait BlacklistCachedPort: Send + Sync + Debug {
    async fn add(&self, jti: &str, ttl: i64) -> UserResult<()>;
    async fn verify(&self, query: &GetJtiExistsQuery) -> UserResult<bool>;
}

pub struct GetJtiExistsQuery {
    pub jti: String,
}
