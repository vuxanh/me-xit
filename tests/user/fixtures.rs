use sqlx::postgres::PgPoolOptions;

pub const CREATE_USER: &str = r"
INSERT INTO users (id, username, password_hash, created_at)
VALUES
('e16a6dfe-4696-4169-9bdb-a52b1d4acb15', 'hello', '$argon2id$v=19$m=19456,t=2,p=1$mjiCrWv/w4JQfpza3S3JCw$+5ZaldBNSMj67WzDstF2xISFUOG4GAQRyHAUoKrvwsQ', '2026-11-11 04:13:14+00')
ON CONFLICT DO NOTHING;
";

pub const CREATE_USER_INVALID_HASH: &str = r"
INSERT INTO users (id, username, password_hash, created_at)
VALUES
('3021208e-7371-4d6e-9cdc-670a9b09f955', 'hello_invalid_hash', '$argon2id$v=19$m=4,t=2,p=1$mjiCrWv/w4JQfpza3S3JCw$+5ZaldBNSMj67WzDstF2xISFUOG4GAQRyHAUoKrvwsQ', '2026-11-11 04:13:14+00')
ON CONFLICT DO NOTHING;
";

pub async fn seed_data(db_url: &str) -> Result<(), std::io::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(25)
        .connect(&db_url)
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    sqlx::query(CREATE_USER).execute(&pool).await.unwrap();

    sqlx::query(CREATE_USER_INVALID_HASH)
        .execute(&pool)
        .await
        .unwrap();

    Ok(())
}
