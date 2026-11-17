use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use domain::User;

use ports::{
    Authenticator, BlacklistCachedPort, Claim, GetJtiExistsQuery, HasherPort, LoginCommand,
    LoginResponse, LogoutCommand, RegisterCommand, Repository, TokenPort, UserInfoQuery,
    ValidateQuery,
};

use error::{UserError, UserResult};

const JWT_TTL: i64 = 30 * 60; // in seconds

pub struct AuthenticationService {
    repo: Box<dyn Repository>,
    blacklist_repo: Box<dyn BlacklistCachedPort>,
    tokenizer: Box<dyn TokenPort>,
    hasher: Box<dyn HasherPort>,
}

impl AuthenticationService {
    pub fn new<R, B, T, H>(repo: R, blacklist_repo: B, tokenizer: T, hasher: H) -> Self
    where
        R: Repository + 'static,
        B: BlacklistCachedPort + 'static,
        T: TokenPort + 'static,
        H: HasherPort + 'static,
    {
        Self {
            repo: Box::new(repo),
            blacklist_repo: Box::new(blacklist_repo),
            tokenizer: Box::new(tokenizer),
            hasher: Box::new(hasher),
        }
    }
}

#[async_trait]
impl Authenticator for AuthenticationService {
    async fn handle_register(&self, command: &RegisterCommand) -> UserResult<()> {
        let RegisterCommand { username, password } = command;

        let password_hash = self.hasher.hash(password)?;

        let user = User::new(
            &Uuid::new_v4().to_string(),
            username,
            &password_hash,
            Utc::now().timestamp(),
        )?;

        self.repo.register(&user.try_into()?).await?;

        Ok(())
    }

    async fn handle_login(&self, command: &LoginCommand) -> UserResult<LoginResponse> {
        let LoginCommand { username, password } = command;

        let user = self
            .repo
            .get_user_password_by_username(&UserInfoQuery {
                username: username.to_string(),
            })
            .await?;

        if !self.hasher.verify(password, &user.password_hash)? {
            return Err(UserError::InvalidCredentials("wrong password".to_string()));
        }

        let jti = &Uuid::new_v4().to_string();

        let token = self.tokenizer.generate_token(username, jti, JWT_TTL)?;

        Ok(LoginResponse {
            token,
            ttl: JWT_TTL,
        })
    }

    async fn handle_validate(&self, query: &ValidateQuery) -> UserResult<bool> {
        let ValidateQuery { token } = query;

        let claim: Claim = self.tokenizer.decode_token(token)?;

        self.blacklist_repo
            .verify(&GetJtiExistsQuery { jti: claim.jti })
            .await
    }

    async fn handle_logout(&self, command: &LogoutCommand) -> UserResult<()> {
        let LogoutCommand { token } = command;

        let Claim {
            username: _,
            jti,
            exp,
        } = self.tokenizer.decode_token(token)?;

        self.blacklist_repo.add(&jti, exp).await
    }
}
