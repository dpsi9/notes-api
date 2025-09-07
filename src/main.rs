use actix_cors::Cors;
use actix_web::{App, HttpServer, http, web};
// use std::sync::Arc;
// use tokio::sync::RwLock;

mod config;
mod db;
mod error;
mod middlewares;
mod notes;
mod tracing;
mod types;

use crate::error::ApiError;
use actix_web::middleware;

// pub type NoteStore = Arc<RwLock<Vec<Note>>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let store: NoteStore = Arc::new(RwLock::new(notes));

    dotenvy::dotenv().ok();
    tracing::init_tracing();

    let pool = db::setup_db(&config::get_config().database_url)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let json_config = web::JsonConfig::default()
        .limit(64 * 1024)
        .error_handler(|err, _| ApiError::BadRequest(format!("invalid JSON: {}", err)).into());

    let json_data = web::Data::new(json_config);

    HttpServer::new(move || {
        App::new()
            .wrap(middlewares::RequestId)
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "PUT", "POST", "DELETE"])
                    .allowed_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::CONTENT_TYPE,
                    ])
                    .max_age(3600),
            )
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("X-Frame-Options", "Deny"))
                    .add(("Referrer-Policy", "no-referrer")),
            )
            .app_data(json_data.clone())
            .app_data(web::Data::new(pool.clone()))
            .configure(notes::routes::init_routes)
    })
    .bind(config::get_config().server_addr)?
    .shutdown_timeout(5)
    .run()
    .await
}
