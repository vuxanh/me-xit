use std::env;

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, http::header, web};

use adapters::{UserRoutes, UserRoutesConfig};
use middlewares::Auth;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn start_server() -> std::io::Result<()> {
    let domain = env::var("DOMAIN").unwrap_or("127.0.0.1".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or("4000".to_string())
        .parse()
        .unwrap_or(4000);

    let config = UserRoutesConfig::default()
        .build()
        .map_err(|e| std::io::Error::other(e))?;

    let user_routes = UserRoutes::default(&config)
        .await
        .map_err(|e| std::io::Error::other(e))?;

    Ok(HttpServer::new(move || {
        // TODO: change localhost
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT,
            ])
            .supports_credentials()
            .max_age(3600);

        let auth = Auth::new();

        App::new()
            .route("/", web::get().to(index))
            .configure(|cfg| user_routes.config(cfg))
            .wrap(cors)
            .wrap(auth)
    })
    .bind((domain, port))?
    .run()
    .await?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    start_server().await.map_err(|e| {
        eprintln!("App init failed");
        std::io::Error::other(e.to_string())
    })
}
