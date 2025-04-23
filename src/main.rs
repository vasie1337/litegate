mod db;
mod electrum;
mod routes;
mod sweeper;
mod utils;
use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use std::env;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();
    info!("Starting application");

    // Initialize tracing
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

    // Get port from environment or use default
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse()
        .unwrap();
    info!("Using port: {}", port);

    // Initialize database
    let db_file = env::var("DB_FILE").unwrap_or_else(|_| "payments.db".into());
    info!("Opening database: {}", db_file);
    let db = db::Db::open(&db_file).expect("Failed to open database");

    // Start sweeper
    info!("Starting sweeper");
    sweeper::start(db.clone()).await;

    // Start HTTP server
    info!("Starting HTTP server on 0.0.0.0:{}", port);
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(db.clone()))
            .configure(routes::config)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
