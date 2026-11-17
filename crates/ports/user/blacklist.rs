use async_trait::async_trait;

use error::UserResult;
use mockall::automock;

#[automock]
#[async_trait]
pub trait BlacklistCachedPort: Send + Sync {
    async fn add(&self, jti: &str, ttl: i64) -> UserResult<()>;
    async fn verify(&self, query: &GetJtiExistsQuery) -> UserResult<bool>;
}

pub struct GetJtiExistsQuery {
    pub jti: String,
}
