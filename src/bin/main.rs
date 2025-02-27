use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use prism_be::app::AppState;
use prism_be::config::parse_config;
use prism_be::server::run_server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let state = AppState {
        accounts: Arc::new(Mutex::new(HashMap::new())),
        data: Arc::new(Mutex::new(HashMap::new())),
    };
    tracing::info!("Starting server...");
    let config = parse_config("config.toml").unwrap();
    run_server(state, config).await;
}
