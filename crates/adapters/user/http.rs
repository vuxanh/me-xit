use std::env::{self, VarError};

use actix_web::{
    HttpResponse, Responder,
    cookie::{Cookie, SameSite, time::Duration},
    error::JsonPayloadError,
    http::StatusCode,
    post,
    web::{self, ServiceConfig},
};
use serde::{Deserialize, Serialize};

use error::{UserError, UserResult};
use ports::{
    Authenticator, BlacklistCachedPort, HasherPort, LoginCommand, LoginResponse, LogoutCommand,
    RegisterCommand, Repository, TokenPort,
};

use crate::user::authentication::AuthenticationService;

use super::{
    blacklist::RedisBlacklistCachedRepository, hasher::ArgonHasher, repository::PgRepository,
    tokenizer::JsonWebTokenizer,
};
pub struct UserAppState {
    authentication_service: AuthenticationService,
}

#[derive(Debug, Serialize)]
struct ResponseJson {
    code: u16,
    message: String,
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

#[post("/register")]
async fn handle_register(
    body: web::Json<RegisterRequest>,
    state: web::Data<UserAppState>,
) -> impl Responder {
    let result = state
        .authentication_service
        .handle_register(&RegisterCommand {
            username: body.username.to_string(),
            password: body.password.to_string(),
        })
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(ResponseJson {
            code: StatusCode::OK.as_u16(),
            message: "Login success".to_string(),
        }),
        Err(e) => map_response_err(e),
    }
}

#[derive(Debug, Deserialize)]
struct LoginRequestJson {
    username: String,
    password: String,
}

#[post("/login")]
async fn handle_login(
    body: web::Json<LoginRequestJson>,
    state: web::Data<UserAppState>,
) -> impl Responder {
    let result = state
        .authentication_service
        .handle_login(&LoginCommand {
            username: body.username.to_string(),
            password: body.password.to_string(),
        })
        .await;

    match result {
        Ok(LoginResponse { token, ttl }) => {
            let auth_cookie = Cookie::build("auth_token", token)
                .secure(true)
                .http_only(true)
                .same_site(SameSite::Strict)
                .max_age(Duration::seconds(ttl))
                .finish();
            HttpResponse::Ok().cookie(auth_cookie).json(ResponseJson {
                code: StatusCode::OK.as_u16(),
                message: "Login success".to_string(),
            })
        }
        Err(e) => map_response_err(e),
    }
}

#[derive(Debug, Deserialize)]
struct LogoutRequest {
    token: String,
}

#[post("/logout")]
async fn handle_logout(
    body: web::Json<LogoutRequest>,
    state: web::Data<UserAppState>,
) -> impl Responder {
    let result = state
        .authentication_service
        .handle_logout(&LogoutCommand {
            token: body.token.to_string(),
        })
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(ResponseJson {
            code: StatusCode::OK.as_u16(),
            message: "Logout success".to_string(),
        }),
        Err(e) => map_response_err(e),
    }
}

pub struct UserServicePorts<R, B, T, H>
where
    R: Repository,
    B: BlacklistCachedPort,
    T: TokenPort,
    H: HasherPort,
{
    pub repo: R,
    pub blacklist_repo: B,
    pub hasher: H,
    pub tokenizer: T,
}

#[derive(Clone)]
pub struct UserRoutes {
    state: web::Data<UserAppState>,
}

impl UserRoutes {
    pub async fn new<R, B, T, H>(ports: UserServicePorts<R, B, T, H>) -> UserResult<Self>
    where
        R: Repository + 'static,
        B: BlacklistCachedPort + 'static,
        T: TokenPort + 'static,
        H: HasherPort + 'static,
    {
        let authentication_service = AuthenticationService::new(
            ports.repo,
            ports.blacklist_repo,
            ports.tokenizer,
            ports.hasher,
        );

        let state = web::Data::new(UserAppState {
            authentication_service,
        });

        Ok(Self { state })
    }

    pub async fn default() -> UserResult<Self> {
        let database_url = env::var("DATABASE_URL").map_err(|e| map_var_err(e, "DATABASE_URL"))?;
        let max_connection = env::var("DATABASE_MAX_CONNECTION")
            .unwrap_or("25".to_string())
            .parse()
            .unwrap_or(25);

        let repo = PgRepository::new(&database_url, max_connection).await?;

        let redis_url =
            env::var("REDIS_BLACKLIST_URL").map_err(|e| map_var_err(e, "REDIS_BLACKLIST_URL"))?;

        let blacklist_repo = RedisBlacklistCachedRepository::new(&redis_url).await?;

        let secret = env::var("JWT_SECRET").map_err(|e| map_var_err(e, "JWT_SECRET"))?;

        let tokenizer = JsonWebTokenizer::new(secret.as_bytes());

        let hasher = ArgonHasher::new();

        let authentication_service =
            AuthenticationService::new(repo, blacklist_repo, tokenizer, hasher);

        let state = web::Data::new(UserAppState {
            authentication_service,
        });

        Ok(Self { state })
    }

    pub fn config(&self, cfg: &mut ServiceConfig) {
        let json_config = web::JsonConfig::default()
            .limit(4096)
            .error_handler(|err, _req| {
                let response = map_response_err(map_json_payload_err(&err));
                actix_web::error::InternalError::from_response(err, response).into()
            });

        cfg.app_data(self.state.clone()).service(
            web::scope("/user")
                .app_data(json_config)
                .service(handle_register)
                .service(handle_login)
                .service(handle_logout),
        );
    }
}

fn map_var_err(e: VarError, key: &str) -> UserError {
    UserError::Config(format!("{} for key: {}", e.to_string(), key.to_string()))
}

fn map_json_payload_err(e: &JsonPayloadError) -> UserError {
    UserError::InvalidToken(e.to_string())
}

fn map_response_err(e: UserError) -> HttpResponse {
    eprintln!("{}", e.to_string());

    let status_code = match e {
        UserError::InvalidCredentials(_) | UserError::MissingToken | UserError::InvalidToken(_) => {
            StatusCode::UNAUTHORIZED
        }
        UserError::AccountLocked => StatusCode::FORBIDDEN,
        UserError::AlreadyExists => StatusCode::CONFLICT,
        UserError::DatabaseError(e) => match e.downcast::<sqlx::error::Error>() {
            Ok(sqlx_err) => match *sqlx_err {
                sqlx::Error::RowNotFound => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        },
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    let message = match status_code {
        StatusCode::UNAUTHORIZED => "Access denied",
        StatusCode::FORBIDDEN => "Resource Forbidden",
        StatusCode::CONFLICT => "Resource already exists",
        _ => "Internal server error",
    };

    HttpResponse::build(status_code).json(ResponseJson {
        code: status_code.as_u16(),
        message: message.to_string(),
    })
}

#[cfg(test)]
mod test {
    use ports::Authenticator;

    use crate::UserRoutes;

    #[actix_web::test]
    async fn default_works() {
        let routes = UserRoutes::default().await.unwrap();

        let result = routes
            .state
            .authentication_service
            .handle_login(&ports::LoginCommand {
                username: "hello".to_string(),
                password: "world".to_string(),
            })
            .await;

        match result {
            Ok(_) => {}
            Err(e) => panic!("{}", e.to_string()),
        }
    }
}
