use std::sync::Arc;

use prism_client::SigningKey;
use prism_prover::Prover;

// Application state
#[derive(Clone)]
pub struct AppState {
    pub prover: Arc<Prover>,
    pub service_id: String,
    pub service_sk: SigningKey,
}
