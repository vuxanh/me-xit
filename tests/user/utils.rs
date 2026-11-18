use std::sync::Arc;

use containers::{Container, setup_postgres_container, setup_redis_container};
// use redis::{AsyncCommands, SetOptions};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::OnceCell;

use crate::user::fixtures::CREATE_USER;

pub async fn init_redis() -> Result<(Container, String), std::io::Error> {
    let (redis, url) = setup_redis_container().await.unwrap();

    println!("Done setup redis");

    Ok((redis, url))
}

pub async fn init_postgres() -> Result<(Container, String), std::io::Error> {
    let (postgres, url) = setup_postgres_container().await.unwrap();

    // Seed db with one user
    let pool = PgPoolOptions::new()
        .max_connections(25)
        .connect(&url)
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    sqlx::query(CREATE_USER).execute(&pool).await.unwrap();

    println!("Done setup postgres");

    Ok((postgres, url))
}

pub struct TestEnv {
    _redis: Container,
    _postgres: Container,
    pub redis_url: String,
    pub postgres_url: String,
}

static TEST_ENV: OnceCell<Arc<TestEnv>> = OnceCell::const_new();

pub async fn setup_test() -> Result<Arc<TestEnv>, std::io::Error> {
    let result = TEST_ENV
        .get_or_try_init(|| async {
            let (_redis, redis_url) = init_redis().await.map_err(|e| std::io::Error::other(e))?;

            let (_postgres, postgres_url) = init_postgres()
                .await
                .map_err(|e| std::io::Error::other(e))?;

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
