use async_trait::async_trait;
use sqlx::{
    PgPool, Row,
    postgres::{PgPoolOptions, PgRow},
};

use crate::user::queries::CREATE_USER;

use super::queries::SELECT_USER_PASSWORD_BY_USERNAME;
use error::{UserError, UserResult};
use ports::{GetUserPasswordDTO, RegisterUserCommandDTO, Repository, UserInfoQuery};

pub struct PgRepository {
    pool: PgPool,
}

impl PgRepository {
    pub async fn new(database_url: &str, max_connection: u32) -> UserResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connection)
            .connect(&database_url)
            .await
            .map_err(map_db_err)?;

        Ok(Self { pool })
    }

    pub async fn from_pool(pool: PgPool) -> UserResult<Self> {
        Ok(Self { pool })
    }
}

#[async_trait]
impl Repository for PgRepository {
    async fn register(&self, command: &RegisterUserCommandDTO) -> UserResult<()> {
        let RegisterUserCommandDTO {
            id,
            username,
            password_hash,
            created_at,
        } = command;

        sqlx::query(CREATE_USER)
            .bind(id)
            .bind(username)
            .bind(password_hash)
            .bind(created_at)
            .execute(&self.pool)
            .await
            .map_err(map_db_err)?;

        Ok(())
    }

    async fn get_user_password_by_username(
        &self,
        query: &UserInfoQuery,
    ) -> UserResult<GetUserPasswordDTO> {
        let UserInfoQuery { username } = query;
        let password = sqlx::query(SELECT_USER_PASSWORD_BY_USERNAME)
            .bind(username)
            .map(|row: PgRow| GetUserPasswordDTO {
                password_hash: row.get("password_hash"),
            })
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;

        Ok(password)
    }
}

fn map_db_err(e: sqlx::error::Error) -> UserError {
    if e.as_database_error()
        .map(|e| e.is_unique_violation())
        .unwrap_or(false)
    {
        return UserError::AlreadyExists;
    }

    UserError::DatabaseError(Box::new(e))
}
