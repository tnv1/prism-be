use std::net::SocketAddr;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::config::Config;

#[derive(Deserialize, Serialize, Debug)]
struct CreateAccountRequest {
    wallet_address: String,
    pub_key: String,
    signature: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct AddKeyRequest {
    wallet_address: String,
    pub_key: String,
    signature: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct AddDataRequest {
    wallet_address: String,
    data: String,
    data_signature: String,
}

#[derive(Serialize)]
struct Response {
    message: String,
    status: String,
}

// Verify signature (placeholder)
fn verify_signature(_message: &str, _signature: &str, _pub_key: &str) -> bool {
    // Implement your signature verification logic here
    true // For demo purposes
}

pub async fn run_server(app_state: AppState, config: Config) {
    // Build the router
    let app = Router::new()
        .route("/v1/account/create", post(create_account))
        .route("/v1/account/add_key", post(add_key))
        .route("/v1/account/add_data", post(add_data))
        .with_state(app_state);

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], config.server.port));
    tracing::info!("Server running on {}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}

// Handlers
async fn create_account(
    State(state): State<AppState>,
    Json(req): Json<CreateAccountRequest>,
) -> impl IntoResponse {
    let message = format!("{}:{}", &req.wallet_address, &req.pub_key);

    if !verify_signature(&message, &req.signature, &req.pub_key) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(Response {
                message: "Invalid signature".to_string(),
                status: "error".to_string(),
            }),
        );
    }

    let mut accounts = state.accounts.lock().unwrap();

    if accounts.contains_key(&req.wallet_address) {
        return (
            StatusCode::CONFLICT,
            Json(Response {
                message: "Account already exists".to_string(),
                status: "error".to_string(),
            }),
        );
    }

    accounts.insert(req.wallet_address.clone(), vec![req.pub_key.clone()]);

    (
        StatusCode::OK,
        Json(Response {
            message: "Account created successfully".to_string(),
            status: "success".to_string(),
        }),
    )
}

async fn add_key(
    State(state): State<AppState>,
    Json(req): Json<AddKeyRequest>,
) -> impl IntoResponse {
    let message = format!("{}:{}", &req.wallet_address, &req.pub_key);

    if !verify_signature(&message, &req.signature, &req.pub_key) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(Response {
                message: "Invalid signature".to_string(),
                status: "error".to_string(),
            }),
        );
    }

    let mut accounts = state.accounts.lock().unwrap();

    match accounts.get_mut(&req.wallet_address) {
        Some(keys) => {
            if keys.contains(&req.pub_key) {
                return (
                    StatusCode::CONFLICT,
                    Json(Response {
                        message: "Key already exists".to_string(),
                        status: "error".to_string(),
                    }),
                );
            }
            keys.push(req.pub_key.clone());
            (
                StatusCode::OK,
                Json(Response {
                    message: "Key added successfully".to_string(),
                    status: "success".to_string(),
                }),
            )
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(Response {
                message: "Account not found".to_string(),
                status: "error".to_string(),
            }),
        ),
    }
}

async fn add_data(
    State(state): State<AppState>,
    Json(req): Json<AddDataRequest>,
) -> impl IntoResponse {
    let message = format!("{}:{}", &req.wallet_address, &req.data);

    if !verify_signature(&message, &req.data_signature, "") {
        return (
            StatusCode::UNAUTHORIZED,
            Json(Response {
                message: "Invalid signature".to_string(),
                status: "error".to_string(),
            }),
        );
    }

    let accounts = state.accounts.lock().unwrap();
    if !accounts.contains_key(&req.wallet_address) {
        return (
            StatusCode::NOT_FOUND,
            Json(Response {
                message: "Account not found".to_string(),
                status: "error".to_string(),
            }),
        );
    }

    let mut data_store = state.data.lock().unwrap();
    let data_vec = data_store.entry(req.wallet_address.clone()).or_insert_with(Vec::new);
    data_vec.push(req.data.clone());

    (
        StatusCode::OK,
        Json(Response {
            message: "Data added successfully".to_string(),
            status: "success".to_string(),
        }),
    )
}
