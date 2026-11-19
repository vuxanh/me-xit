use std::env;

use actix_web::{App, HttpResponse, HttpServer, Responder, web};

use adapters::{UserRoutes, UserRoutesConfig};

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn start_server() -> std::io::Result<()> {
    let domain = env::var("DOMAIN").unwrap_or("127.0.0.1".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or("3000".to_string())
        .parse()
        .unwrap_or(3000);

    let config = UserRoutesConfig::default()
        .build()
        .map_err(|e| std::io::Error::other(e))?;

    let user_routes = UserRoutes::default(&config)
        .await
        .map_err(|e| std::io::Error::other(e))?;

    Ok(HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(index))
            .configure(|cfg| user_routes.config(cfg))
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
