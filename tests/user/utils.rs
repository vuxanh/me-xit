use std::sync::Arc;

use containers::{
    Container, setup_postgres_container, setup_redis_container, setup_redis_readonly_container,
};
use tokio::sync::OnceCell;

use crate::user::fixtures::seed_data;

pub async fn init_redis() -> Result<(Container, String), std::io::Error> {
    println!("Setting up redis...");

    let (redis, url) = setup_redis_container().await.unwrap();

    println!("Done setup redis\n");

    Ok((redis, url))
}

pub async fn init_redis_readonly() -> Result<(Container, String), std::io::Error> {
    println!("Setting up readonly redis...");

    let (redis, url) = setup_redis_readonly_container().await.unwrap();

    println!("Done setup readonly redis\n");

    Ok((redis, url))
}

pub async fn init_postgres() -> Result<(Container, String), std::io::Error> {
    println!("Setting up postgres...");

    let (postgres, url) = setup_postgres_container().await.unwrap();

    println!("Seeding db...");
    seed_data(&url).await?;

    println!("Done setup postgres\n");

    Ok((postgres, url))
}

pub struct TestEnv {
    _redis: Container,
    _redis_readonly: Container,
    _postgres: Container,
    pub redis_url: String,
    pub redis_readonly_url: String,
    pub postgres_url: String,
}

static TEST_ENV: OnceCell<Arc<TestEnv>> = OnceCell::const_new();

pub async fn setup_test() -> Result<Arc<TestEnv>, std::io::Error> {
    let result = TEST_ENV
        .get_or_try_init(|| async {
            let (_redis, redis_url) = init_redis().await.map_err(|e| std::io::Error::other(e))?;

            let (_redis_readonly, redis_readonly_url) = init_redis_readonly()
                .await
                .map_err(|e| std::io::Error::other(e))?;

            let (_postgres, postgres_url) = init_postgres()
                .await
                .map_err(|e| std::io::Error::other(e))?;

            Ok::<_, std::io::Error>(Arc::new(TestEnv {
                _redis,
                _postgres,
                redis_url,
                postgres_url,
                _redis_readonly,
                redis_readonly_url,
            }))
        })
        .await?;

    Ok(result.clone())
}
