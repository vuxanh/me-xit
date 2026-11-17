use error::UserResult;
use macros::Getters;

use crate::user::username::Username;

#[derive(Debug, Getters)]
pub struct User {
    id: String,
    username: Username,
    password_hash: String,
    created_at: i64,
}

impl User {
    pub fn new(id: &str, username: &str, password_hash: &str, created_at: i64) -> UserResult<Self> {
        Ok(Self {
            id: id.to_string(),
            username: Username::new(username)?,
            password_hash: password_hash.to_string(),
            created_at: created_at,
        })
    }
}
