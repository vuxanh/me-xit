use std::env;

use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn init_db_pool() -> Result<PgPool, sqlx::error::Error> {
    let url = env::var("DATABASE_URL").map_err(|e| sqlx::error::Error::Decode(Box::new(e)))?;
    let max_connection: u32 = env::var("DATABASE_MAX_CONNECTION")
        .unwrap_or("25".to_string())
        .parse()
        .map_err(|e| sqlx::error::Error::Decode(Box::new(e)))?;

    let pool = PgPoolOptions::new()
        .max_connections(max_connection)
        .connect(&url)
        .await?;

    Ok(pool)
}
