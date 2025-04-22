use once_cell::sync::Lazy;

pub static MAIN_ADDRESS: Lazy<String> =
    Lazy::new(|| std::env::var("MAIN_ADDRESS").expect("MAIN_ADDRESS"));

pub mod utils;
pub mod db;
pub mod electrum;
pub mod routes;
pub mod sweeper;
