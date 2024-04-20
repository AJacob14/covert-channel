use std::{fs, sync::{Arc, Mutex}};

use axum::{
    extract::State, http::{HeaderMap, StatusCode}, routing::post, Router
};
use base64::{engine::general_purpose, Engine};
use chrono::Local;

struct AppState {
    data: String
}

impl AppState {
    fn new() -> Self {
        AppState { data: String::new() }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let shared_state = Arc::new(Mutex::new(AppState::new()));
    let app = Router::new()
        .route("/embeddings", post(create_embeddings))
        .route("/chat/completions", post(chat))
        .route("/batches", post(batches))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await;
    if listener.is_err() {
        eprintln!("Failed to start server\nReason: {}", listener.unwrap_err());
        return;
    }
    let listener = listener.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_embeddings(State(state): State<Arc<Mutex<AppState>>>) -> StatusCode {
    let state = state.lock();
    let mut state = state.unwrap();
    state.data = String::new();
    StatusCode::CREATED
}

async fn chat(headers: HeaderMap, State(state): State<Arc<Mutex<AppState>>>) -> StatusCode {
    let state = state.lock();
    let mut state = state.unwrap();
    if !headers.contains_key("Authorization"){
        return StatusCode::UNAUTHORIZED;
    }

    let bearer = headers.get("Authorization");
    if bearer.is_none() {
        return StatusCode::UNAUTHORIZED;
    }

    let bearer = bearer.unwrap();
    let result = bearer.to_str();
    if result.is_err() {
        eprintln!("Failed to read headers\nReason: {}", result.unwrap_err());
        return StatusCode::BAD_REQUEST;
    }

    let value = result.unwrap();
    let mut values = value.split(' ');
    let result = values.nth(1);
    if result.is_none() {
        eprintln!("Failed to extract headers");
        return StatusCode::BAD_REQUEST;
    }

    let data = result.unwrap();
    state.data += data;

    StatusCode::OK
}

async fn batches(State(state): State<Arc<Mutex<AppState>>>) -> StatusCode {
    let state = state.lock();
    let state = state.unwrap();
    let now = Local::now();
    let name = now.format("%Y%m%d%H%M%S").to_string();
    let name = format!("{}.bin", name);
    let result = general_purpose::STANDARD.decode(&state.data);
    if result.is_err() {
        eprintln!("Failed to write data\nReason: {}", result.unwrap_err());
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    let data = result.unwrap();
    let result = fs::write(name, data);
    if result.is_err() {
        eprintln!("Failed to write data\nReason: {}", result.unwrap_err());
        StatusCode::INTERNAL_SERVER_ERROR
    }
    else {
        StatusCode::OK
    }
}