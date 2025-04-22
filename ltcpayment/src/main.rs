mod db;
mod electrum;
mod routes;
mod sweeper;
mod utils;
use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    println!("[Bootstrap] starting");
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse()
        .unwrap();
    let db = db::Db::open(&env::var("DB_FILE").unwrap_or_else(|_| "payments.db".into()));
    sweeper::start(db.clone()).await;
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(db.clone()))
            .configure(routes::config)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
