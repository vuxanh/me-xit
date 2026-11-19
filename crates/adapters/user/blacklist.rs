use error::{UserError, UserResult};
use ports::{BlacklistCachedPort, GetJtiExistsQuery};

use async_trait::async_trait;
use chrono::Utc;
use redis::{AsyncCommands, Client, RedisError, SetOptions, aio::MultiplexedConnection};

#[derive(Debug)]
pub struct RedisBlacklistCachedRepository {
    pool: MultiplexedConnection,
}

impl RedisBlacklistCachedRepository {
    pub async fn new(url: &str) -> UserResult<Self> {
        let pool = Client::open(url)
            .map_err(map_redis_err)?
            .get_multiplexed_async_connection()
            .await
            .map_err(map_redis_err)?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl BlacklistCachedPort for RedisBlacklistCachedRepository {
    async fn add(&self, jti: &str, ttl: i64) -> UserResult<()> {
        let revoke_timestamp = Utc::now().timestamp();

        let options = SetOptions::default()
            .with_expiration(redis::SetExpiry::EX(ttl.try_into().unwrap_or_default()));

        let key = format!("bl:jwt:{}", jti);

        let result: () = self
            .pool
            .clone()
            .set_options(key, revoke_timestamp, options)
            .await
            .map_err(map_redis_err)?;

        Ok(result)
    }

    async fn verify(&self, query: &GetJtiExistsQuery) -> UserResult<bool> {
        let GetJtiExistsQuery { jti } = query;

        let key = format!("bl:jwt:{}", jti);

        let result = self.pool.clone().exists(key).await.map_err(map_redis_err)?;

        Ok(result)
    }
}

fn map_redis_err(e: RedisError) -> UserError {
    UserError::CacheError(Box::new(e))
}
