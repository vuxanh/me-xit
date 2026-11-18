use std::sync::Arc;

// use redis::{AsyncCommands, SetOptions};
use sqlx::postgres::PgPoolOptions;
use testcontainers::{ContainerAsync, GenericImage, ImageExt, core::WaitFor, runners::AsyncRunner};
use tokio::sync::OnceCell;

use crate::user::fixtures::CREATE_USER;

pub async fn init_redis() -> Result<(ContainerAsync<GenericImage>, String), std::io::Error> {
    let redis = GenericImage::new("redis", "8.10-alpine")
        .with_wait_for(WaitFor::message_on_stdout(
            "Ready to accept connections tcp",
        ))
        .start()
        .await
        .map_err(|e| std::io::Error::other(e))?;

    let port = redis
        .get_host_port_ipv4(6379)
        .await
        .map_err(|e| std::io::Error::other(e))?;

    let url = format!("redis://127.0.0.1:{port}");

    println!("Done setup redis");

    Ok((redis, url))
}

pub async fn init_postgres() -> Result<(ContainerAsync<GenericImage>, String), std::io::Error> {
    let postgres = GenericImage::new("postgres", "18.7-alpine")
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .start()
        .await
        .map_err(|e| std::io::Error::other(e))?;

    let port = postgres
        .get_host_port_ipv4(5432)
        .await
        .map_err(|e| std::io::Error::other(e))?;

    let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres?sslmode=disable");

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
    _redis: ContainerAsync<GenericImage>,
    _postgres: ContainerAsync<GenericImage>,
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
