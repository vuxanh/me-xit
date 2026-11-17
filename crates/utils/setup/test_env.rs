use std::sync::Arc;

use testcontainers::ContainerAsync;
use testcontainers_modules::{postgres::Postgres, redis::Redis};
use tokio::sync::OnceCell;

use crate::{init_postgres, init_redis};

pub struct TestEnv {
    _redis: ContainerAsync<Redis>,
    _postgres: ContainerAsync<Postgres>,
    pub redis_url: String,
    pub postgres_url: String,
}

static TEST_ENV: OnceCell<Arc<TestEnv>> = OnceCell::const_new();

pub async fn setup_test() -> Result<Arc<TestEnv>, std::io::Error> {
    let result = TEST_ENV
        .get_or_try_init(|| async {
            let (_postgres, postgres_url) = init_postgres()
                .await
                .map_err(|e| std::io::Error::other(e))?;
            let (_redis, redis_url) = init_redis().await.map_err(|e| std::io::Error::other(e))?;

            Ok::<_, std::io::Error>(Arc::new(TestEnv {
                _redis,
                _postgres,
                redis_url,
                postgres_url,
            }))
        })
        .await?;

    Ok(result.clone())
}
