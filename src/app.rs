use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Application state
#[derive(Clone)]
pub struct AppState {
    pub accounts: Arc<Mutex<HashMap<String, Vec<String>>>>,
    pub data: Arc<Mutex<HashMap<String, Vec<String>>>>,
}
