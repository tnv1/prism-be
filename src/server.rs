use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use prism_client::{SignatureBundle, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::config::Config;
use crate::ops::{add_data, add_key, create_account, register_service};

#[derive(Deserialize, Serialize, Debug)]
struct CreateAccountRequest {
    wallet_address: String,
    pub_key: String,
    signature: SignatureBundle,
}

#[derive(Deserialize, Serialize, Debug)]
struct AddKeyRequest {
    wallet_address: String,
    pub_key: String,
    signature: SignatureBundle,
}

#[derive(Deserialize, Serialize, Debug)]
struct AddDataRequest {
    wallet_address: String,
    data: Vec<u8>,
    data_signature: SignatureBundle,
    signature: SignatureBundle,
}

#[derive(Serialize)]
struct AccountResponse {
    id: String,
}

#[derive(Serialize)]
struct RegisterServiceResponse {
    message: String,
}

pub async fn run_server(app_state: AppState, config: Config) {
    // Wrap app_state in Arc
    let app_state = Arc::new(app_state);

    // Build the router
    let app = Router::new()
        .route("/v1/health", get(health_check_handler))
        .route("/v1/account/register", post(register_service_handler))
        .route("/v1/account/create", post(create_account_handler))
        .route("/v1/account/add_key", post(add_key_handler))
        .route("/v1/account/add_data", post(add_data_handler))
        .with_state(app_state);

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Server running on {}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}

// Handlers

// Health check
async fn health_check_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

// Register service
async fn register_service_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let state = state.clone();
    register_service(state).await.unwrap();

    (StatusCode::OK, Json(RegisterServiceResponse { message: "Service registered".to_string() }))
}

async fn create_account_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAccountRequest>,
) -> impl IntoResponse {
    let state = state.clone();
    let account = create_account(state, req.wallet_address, req.signature).await.unwrap();

    (StatusCode::OK, Json(AccountResponse { id: account.id().to_string() }))
}

async fn add_key_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddKeyRequest>,
) -> impl IntoResponse {
    let state = state.clone();
    let new_key = VerifyingKey::try_from(req.pub_key).unwrap();
    let account = add_key(state, req.wallet_address, new_key, req.signature).await.unwrap();

    (StatusCode::OK, Json(AccountResponse { id: account.id().to_string() }))
}

async fn add_data_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddDataRequest>,
) -> impl IntoResponse {
    let state = state.clone();
    let account = add_data(state, req.wallet_address, req.data, req.data_signature, req.signature)
        .await
        .unwrap();

    (StatusCode::OK, Json(AccountResponse { id: account.id().to_string() }))
}
