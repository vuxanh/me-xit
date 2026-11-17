use error::UserResult;
use mockall::automock;

#[automock]
pub trait HasherPort: Send + Sync {
    fn hash(&self, password: &str) -> UserResult<String>;
    fn verify(&self, password: &str, password_hash: &str) -> UserResult<bool>;
}
