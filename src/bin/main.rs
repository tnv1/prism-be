use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use keystore_rs::{KeyChain, KeyStore};
use prism_be::app::AppState;
use prism_be::config::parse_config;
use prism_be::server::run_server;
use prism_client::SigningKey;
use prism_da::memory::InMemoryDataAvailabilityLayer;
use prism_prover::Prover;
use prism_storage::inmemory::InMemoryDatabase;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = parse_config("config.toml").unwrap();

    let db = InMemoryDatabase::new();
    let (da_layer, _, _) = InMemoryDataAvailabilityLayer::new(5);

    let keystore_sk = KeyChain
        .get_or_create_signing_key(&config.service_id)
        .map_err(|e| anyhow!("Error getting key from store: {}", e))?;

    let sk = SigningKey::Ed25519(Box::new(keystore_sk.clone()));

    let cfg = Config {
        prover: true,
        batcher: true,
        webserver: WebServerConfig { enabled: true, host: "127.0.0.1".to_string(), port: 50524 },
        signing_key: sk.clone(),
        verifying_key: sk.verifying_key(),
        start_height: 1,
    };

    let prover = Arc::new(
        Prover::new(
            Arc::new(Box::new(db)),
            Arc::new(da_layer) as Arc<dyn DataAvailabilityLayer>,
            &cfg,
        )
        .unwrap(),
    );

    let runner = prover.clone();
    let runner_handle = spawn(async move {
        debug!("starting prover");
        if let Err(e) = runner.run().await {
            log::error!("Error occurred while running prover: {:?}", e);
        }
    });

    tokio::select! {
        _ = runner_handle => {
            println!("Prover runner task completed");
        }
    }

    let state = AppState { prover, service_id, service_sk };
    tracing::info!("Starting server...");

    run_server(state, config).await;
}
