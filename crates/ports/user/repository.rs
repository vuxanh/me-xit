use std::str::FromStr;

use async_trait::async_trait;

use chrono::{DateTime, Utc};
use domain::User;
use error::{UserError, UserResult};
use mockall::automock;
use uuid::Uuid;

pub struct GetUserPasswordDTO {
    pub password_hash: String,
}

pub struct RegisterUserCommandDTO {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[automock]
#[async_trait]
pub trait Repository: Send + Sync {
    async fn get_user_password_by_username(
        &self,
        query: &UserInfoQuery,
    ) -> UserResult<GetUserPasswordDTO>;

    async fn register(&self, command: &RegisterUserCommandDTO) -> UserResult<()>;
}

pub struct UserInfoQuery {
    pub username: String,
}

impl TryFrom<User> for RegisterUserCommandDTO {
    type Error = UserError;
    fn try_from(user: User) -> UserResult<Self> {
        let Some(created_at) = DateTime::from_timestamp(user.created_at(), 0) else {
            return Err(UserError::CodecError(
                "couldn't create user, failed to convert timestamp".to_string(),
            ));
        };

        let id = Uuid::from_str(user.id())
            .map_err(|e: uuid::Error| UserError::CodecError(e.to_string()))?;

        Ok(Self {
            id,
            username: user.username().to_string(),
            password_hash: user.password_hash().to_string(),
            created_at,
        })
    }
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, Utc};
    use domain::User;
    use error::UserError;
    use uuid::Uuid;

    use crate::RegisterUserCommandDTO;

    #[test]
    fn try_from_works() {
        let id = Uuid::new_v4();
        let username = "username";
        let password = "password";
        let created_at = Utc::now().timestamp();

        let user = User::new(&id.to_string(), username, password, created_at).unwrap();
        let user = RegisterUserCommandDTO::try_from(user).unwrap();

        assert_eq!(user.id, id);
        assert_eq!(user.username, username);
        assert_eq!(user.password_hash, password);
        assert_eq!(
            user.created_at,
            DateTime::from_timestamp(created_at, 0).unwrap()
        );
    }

    #[test]
    fn try_from_fails() {
        let id = Uuid::new_v4();
        let username = "username";
        let password = "password";
        let created_at = i64::MAX;

        let user = User::new(&id.to_string(), username, password, created_at).unwrap();

        let user = RegisterUserCommandDTO::try_from(user);

        assert!(user.is_err());

        match user {
            Ok(_) => {
                panic!("expected error")
            }
            Err(e) => match e {
                UserError::CodecError(_) => {}
                _ => {
                    panic!("expected codec error")
                }
            },
        }
    }
}
