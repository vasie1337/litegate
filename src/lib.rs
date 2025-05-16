use once_cell::sync::Lazy;
use tracing::{error, info};

pub static MAIN_ADDRESS: Lazy<String> = Lazy::new(|| match std::env::var("MAIN_ADDRESS") {
    Ok(addr) => {
        info!("Loaded MAIN_ADDRESS from environment");
        addr
    }
    Err(e) => {
        error!("Failed to load MAIN_ADDRESS: {}", e);
        panic!("MAIN_ADDRESS environment variable is required");
    }
});

pub mod db;
pub mod electrum;
pub mod routes;
pub mod sweeper;
pub mod utils;
pub mod webhook;