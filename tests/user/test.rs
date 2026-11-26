#[cfg(test)]
mod test {
    use std::sync::Arc;

    use actix_web::{
        App,
        http::{StatusCode, header::ContentType},
        test,
    };
    use adapters::{
        ArgonHasher, JsonWebTokenizer, PgRepository, RedisBlacklistCachedRepository, UserRoutes,
        UserServicePorts,
    };
    use error::UserError;
    use ports::{Claim, MockHasherPort, MockRepository, MockTokenPort};
    use serde::Deserialize;

    use crate::user::utils::{TestEnv, setup_test};

    #[derive(Debug, Deserialize)]
    struct ResponseJson {
        code: u16,
        message: String,
    }

    //========================
    //  Login tests         //
    //========================
    //
    //
    /// Success flow
    /// with username and password
    /// responds with http_only cookie
    /// status code OK
    /// login success
    #[tokio::test]
    async fn login_route_works() {
        let env = setup_test().await.unwrap();

        let UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        } = init_services(env).await;

        let routes = UserRoutes::new(UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        })
        .await
        .unwrap();

        let app = test::init_service(App::new().configure(|cfg| routes.config(cfg))).await;

        let req = test::TestRequest::post()
            .uri("/user/login")
            .insert_header(ContentType::json())
            .set_json(serde_json::json!(
                {
                    "username": "hello",
                    "password": "world"
                }
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;

        let auth_cookie = resp
            .response()
            .cookies()
            .find(|c| c.name() == "auth_token")
            .unwrap();

        println!("auth: {}", auth_cookie);
        assert!(!auth_cookie.value().is_empty());

        let status = resp.status();
        let body: ResponseJson =
            serde_json::from_slice(&test::read_body(resp).await.to_vec()).unwrap();

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.code, 200);
        assert_eq!(body.message, "Login success");
    }

    /// Fails with invalid password
    ///
    /// should respond with 401
    /// access denied
    #[tokio::test]
    async fn login_route_fails_invalid_password() {
        let env = setup_test().await.unwrap();

        let UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        } = init_services(env).await;

        let routes = UserRoutes::new(UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        })
        .await
        .unwrap();

        let app = test::init_service(App::new().configure(|cfg| routes.config(cfg))).await;

        let req = test::TestRequest::post()
            .uri("/user/login")
            .insert_header(ContentType::json())
            .set_json(serde_json::json!(
                {
                    "username": "hello",
                    "password": "password"
                }
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;

        let status = resp.status();
        let body: ResponseJson =
            serde_json::from_slice(&test::read_body(resp).await.to_vec()).unwrap();

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body.code, 401);
        assert_eq!(body.message, "Access denied");
    }

    /// Fails with invalid username
    ///
    /// should respond with 401
    /// access denied
    #[tokio::test]
    async fn login_route_fails_invalid_username() {
        let env = setup_test().await.unwrap();

        let UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        } = init_services(env).await;

        let routes = UserRoutes::new(UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        })
        .await
        .unwrap();

        let app = test::init_service(App::new().configure(|cfg| routes.config(cfg))).await;

        let req = test::TestRequest::post()
            .uri("/user/login")
            .insert_header(ContentType::json())
            .set_json(serde_json::json!(
                {
                    "username": "username",
                    "password": "world"
                }
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;

        let status = resp.status();
        let body: ResponseJson =
            serde_json::from_slice(&test::read_body(resp).await.to_vec()).unwrap();

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body.code, 401);
        assert_eq!(body.message, "Access denied");
    }

    /// Fails with repository error
    ///
    /// Read/write/access error from the repository
    /// Should log error out with message
    /// Respond with 401 and hide error message
    /// access denied
    #[tokio::test]
    async fn login_route_fails_with_repo_error() {
        let env = setup_test().await.unwrap();

        let UserServicePorts {
            repo: _,
            blacklist_repo,
            hasher,
            tokenizer,
        } = init_services(env).await;

        let mut repo = MockRepository::default();

        repo.expect_get_user_password_by_username()
            .once()
            .returning(|_| {
                Err(UserError::DatabaseError(Box::new(
                    sqlx::error::Error::RowNotFound,
                )))
            });

        let routes = UserRoutes::new(UserServicePorts {
            repo,
            blacklist_repo,
            tokenizer,
            hasher,
        })
        .await
        .unwrap();

        let app = test::init_service(App::new().configure(|cfg| routes.config(cfg))).await;

        let req = test::TestRequest::post()
            .uri("/user/login")
            .insert_header(ContentType::json())
            .set_json(serde_json::json!(
                {
                    "username": "hello",
                    "password": "world"
                }
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;

        let status = resp.status();
        let body: ResponseJson =
            serde_json::from_slice(&test::read_body(resp).await.to_vec()).unwrap();

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body.code, 401);
        assert_eq!(body.message, "Access denied");
    }

    /// Fails with hasher error
    ///
    /// Error in the hasher.
    /// Decode/verify password errors
    /// Respond with 401 and hide error message
    /// access denied
    #[tokio::test]
    async fn login_route_fails_with_hasher_error() {
        let env = setup_test().await.unwrap();

        let UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        } = init_services(env).await;

        let routes = UserRoutes::new(UserServicePorts {
            repo,
            blacklist_repo,
            tokenizer,
            hasher,
        })
        .await
        .unwrap();

        let app = test::init_service(App::new().configure(|cfg| routes.config(cfg))).await;

        let req = test::TestRequest::post()
            .uri("/user/login")
            .insert_header(ContentType::json())
            .set_json(serde_json::json!(
                {
                    "username": "hello_invalid_hash",
                    "password": "world"
                }
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;

        let status = resp.status();
        let body: ResponseJson =
            serde_json::from_slice(&test::read_body(resp).await.to_vec()).unwrap();

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body.code, 500);
        assert_eq!(body.message, "Internal server error");
    }

    //========================
    // Logout tests         //
    //========================
    //
    //
    /// Success flow
    /// With jwt token in the response body
    /// Respond with success
    /// logout success
    #[tokio::test]
    async fn logout_route_works() {
        let env = setup_test().await.unwrap();

        let UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer: _,
        } = init_services(env).await;

        let mut tokenizer = MockTokenPort::default();

        tokenizer.expect_decode_token().once().returning(|_| {
            Ok(Claim {
                username: "hello".to_string(),
                jti: "mock_jti".to_string(),
                exp: 1,
                role: "user".to_string(),
            })
        });

        let routes = UserRoutes::new(UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        })
        .await
        .unwrap();

        let app = test::init_service(App::new().configure(|cfg| routes.config(cfg))).await;

        let req = test::TestRequest::post()
            .uri("/user/logout")
            .insert_header(ContentType::json())
            .set_json(serde_json::json!(
                {
                    "token": "mock_token"
                }
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        let body: ResponseJson =
            serde_json::from_slice(&test::read_body(resp).await.to_vec()).unwrap();

        assert_eq!(body.message, "Logout success");
        assert_eq!(body.code, 200);
        assert_eq!(status, StatusCode::OK);
    }

    /// Fails when request body contains no token
    ///
    /// Error when deserialize json body
    /// Respond with 401 and hide error message
    /// access denied
    #[tokio::test]
    async fn logout_route_fails_no_token_request() {
        let env = setup_test().await.unwrap();

        let UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer: _,
        } = init_services(env).await;

        let tokenizer = MockTokenPort::default();

        let routes = UserRoutes::new(UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        })
        .await
        .unwrap();

        let app = test::init_service(App::new().configure(|cfg| routes.config(cfg))).await;

        let req = test::TestRequest::post()
            .uri("/user/logout")
            .insert_header(ContentType::json())
            .set_json(serde_json::json!({}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        let body = &test::read_body(resp).await;

        let body: ResponseJson = serde_json::from_slice(body).unwrap();

        assert_eq!(body.message, "Access denied");
        assert_eq!(body.code, 401);
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    /// Fails with blacklist repo cache errors
    ///
    /// Respond with 500 and hide error message
    /// Internal server error
    #[tokio::test]
    async fn logout_route_fails_with_blacklist_repo_error() {
        let env = setup_test().await.unwrap();

        let UserServicePorts {
            repo,
            blacklist_repo: _,
            hasher,
            tokenizer: _,
        } = init_services(env.clone()).await;

        let blacklist_repo = RedisBlacklistCachedRepository::new(&env.redis_readonly_url)
            .await
            .unwrap();

        let mut tokenizer = MockTokenPort::default();

        tokenizer.expect_decode_token().once().returning(|_| {
            Ok(Claim {
                username: "hello".to_string(),
                jti: "mock_jti".to_string(),
                exp: 1,
                role: "User".to_string(),
            })
        });

        let routes = UserRoutes::new(UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        })
        .await
        .unwrap();

        let app = test::init_service(App::new().configure(|cfg| routes.config(cfg))).await;

        let req = test::TestRequest::post()
            .uri("/user/logout")
            .insert_header(ContentType::json())
            .set_json(serde_json::json!(
                {
                    "token": "mock_token"
                }
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        let body: ResponseJson =
            serde_json::from_slice(&test::read_body(resp).await.to_vec()).unwrap();

        assert_eq!(body.message, "Internal server error");
        assert_eq!(body.code, 500);
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    /// Fails with tokenizer errors
    ///
    /// Respond with 500 and hide error message
    /// Internal server error
    #[tokio::test]
    async fn logout_route_fails_with_tokenizer_error() {
        let env = setup_test().await.unwrap();

        let UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        } = init_services(env).await;

        let routes = UserRoutes::new(UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        })
        .await
        .unwrap();

        let app = test::init_service(App::new().configure(|cfg| routes.config(cfg))).await;

        let req = test::TestRequest::post()
            .uri("/user/logout")
            .insert_header(ContentType::json())
            .set_json(serde_json::json!(
                {
                    "token": "mock_token"
                }
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        let body: ResponseJson =
            serde_json::from_slice(&test::read_body(resp).await.to_vec()).unwrap();

        assert_eq!(body.message, "Access denied");
        assert_eq!(body.code, 401);
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    //========================
    // Register tests       //
    //========================
    //
    //
    /// Success flow
    /// Request with username and password
    /// Responds with success
    /// register success
    #[tokio::test]
    async fn register_works() {
        let env = setup_test().await.unwrap();

        let UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        } = init_services(env).await;

        let routes = UserRoutes::new(UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        })
        .await
        .unwrap();

        let app = test::init_service(App::new().configure(|cfg| routes.config(cfg))).await;

        let req = test::TestRequest::post()
            .uri("/user/register")
            .insert_header(ContentType::json())
            .set_json(serde_json::json!(
                {
                    "username": "register_works_hello",
                    "password": "register_works_world"
                }
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;

        let status = resp.status();
        let body: ResponseJson =
            serde_json::from_slice(&test::read_body(resp).await.to_vec()).unwrap();

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.code, 200);
        assert_eq!(body.message, "Login success");
    }

    /// Register fails with duplicated username
    /// Responds with conflict
    /// Resource already exists
    #[tokio::test]
    async fn register_fails_duplicate_username() {
        let env = setup_test().await.unwrap();

        let UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        } = init_services(env).await;

        let routes = UserRoutes::new(UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        })
        .await
        .unwrap();

        let app = test::init_service(App::new().configure(|cfg| routes.config(cfg))).await;

        let req = test::TestRequest::post()
            .uri("/user/register")
            .insert_header(ContentType::json())
            .set_json(serde_json::json!(
                {
                    "username": "hello",
                    "password": "world"
                }
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;

        let status = resp.status();
        let body: ResponseJson =
            serde_json::from_slice(&test::read_body(resp).await.to_vec()).unwrap();

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body.code, 409);
        assert_eq!(body.message, "Resource already exists");
    }

    /// Register fails with hasher error
    /// Responds with 500
    /// Internal server error
    #[tokio::test]
    async fn register_fails_with_hasher_fail() {
        let env = setup_test().await.unwrap();

        let UserServicePorts {
            repo,
            blacklist_repo,
            hasher: _,
            tokenizer,
        } = init_services(env).await;

        let mut hasher = MockHasherPort::default();
        hasher
            .expect_hash()
            .once()
            .returning(|_| Err(UserError::CodecError("error".to_string())));

        let routes = UserRoutes::new(UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        })
        .await
        .unwrap();

        let app = test::init_service(App::new().configure(|cfg| routes.config(cfg))).await;

        let req = test::TestRequest::post()
            .uri("/user/register")
            .insert_header(ContentType::json())
            .set_json(serde_json::json!(
                {
                    "username": "hello",
                    "password": "world"
                }
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;

        let status = resp.status();
        let body: ResponseJson =
            serde_json::from_slice(&test::read_body(resp).await.to_vec()).unwrap();

        assert_eq!(body.message, "Internal server error");
        assert_eq!(body.code, 500);
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    async fn init_services(
        env: Arc<TestEnv>,
    ) -> UserServicePorts<
        PgRepository,
        RedisBlacklistCachedRepository,
        JsonWebTokenizer,
        ArgonHasher<'static>,
    > {
        let repo = PgRepository::new(&env.postgres_url, 25).await.unwrap();

        let blacklist_repo = RedisBlacklistCachedRepository::new(&env.redis_url)
            .await
            .unwrap();

        let tokenizer = JsonWebTokenizer::new("secret".as_bytes());

        let hasher = ArgonHasher::new();

        UserServicePorts {
            repo,
            blacklist_repo,
            hasher,
            tokenizer,
        }
    }
}

/// Clean up
///
/// Stop and delete all containers created by testcontainers
#[cfg(test)]
#[dtor::dtor(unsafe)]
fn cleanup() {
    println!("starting cleanup...");
    if let Ok(out) = std::process::Command::new("podman")
        .args([
            "ps",
            "-aq",
            "--filter",
            "label=org.testcontainers.managed-by=testcontainers",
        ])
        .output()
    {
        let ids: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .map(String::from)
            .collect();

        if !ids.is_empty() {
            println!("removing containers\n{}", ids.join("\n"));
            let _ = std::process::Command::new("podman")
                .args(["rm", "-f"])
                .args(&ids)
                .status();
        }
    }

    println!("removing anonymous volumes...");

    let _ = std::process::Command::new("podman")
        .args(["volume", "prune", "--filter", "anonymous=true", "-f"])
        .status();
}
