use error::UserResult;
use mockall::automock;

use std::fmt::Debug;

#[automock]
pub trait HasherPort: Send + Sync + Debug {
    fn hash(&self, password: &str) -> UserResult<String>;
    fn verify(&self, password: &str, password_hash: &str) -> UserResult<bool>;
}
