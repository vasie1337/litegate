mod db;
mod electrum;
mod routes;
mod sweeper;
mod utils;
mod webhook;
use actix_web::{App, HttpServer};
use actix_cors::Cors;
use dotenvy::dotenv;
use std::env;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    info!("Starting application");

    let subscriber = FmtSubscriber::builder()
        .with_max_level(
            match env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "INFO".into())
                .to_uppercase()
                .as_str()
            {
                "TRACE" => Level::TRACE,
                "DEBUG" => Level::DEBUG,
                "WARN" => Level::WARN,
                "ERROR" => Level::ERROR,
                _ => Level::INFO,
            },
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse()
        .unwrap();
    info!("Using port: {}", port);

    let db_file = env::var("DB_FILE").unwrap_or_else(|_| "payments.db".into());
    info!("Opening database: {}", db_file);
    let db = db::Db::open(&db_file).expect("Failed to open database");

    info!("Starting sweeper");
    sweeper::start(db.clone()).await;

    info!("Starting HTTP server on 0.0.0.0:{}", port);
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600)
            )
            .app_data(actix_web::web::Data::new(db.clone()))
            .configure(routes::config)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}