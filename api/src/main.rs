use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use common::LogEntry;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber;
use ulid::Ulid;

#[derive(Clone)]
struct AppState {
    logs: Arc<RwLock<Vec<LogEntry>>>,
}

#[derive(Debug, serde::Deserialize)]
struct CreateLog {
    message: String,
}

#[tokio::main]
async fn main() {
    // set up logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    let state = AppState {
        logs: Arc::new(RwLock::new(Vec::new())),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/logs", get(list_logs))
        .route("/logs", post(create_log))
        .route("/ping", get(ping))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Bind failed: {addr}");

    tracing::info!("Server running on http://{addr}");

    axum::serve::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello from Chad Log API!"
}

async fn create_log(
    State(state): State<AppState>,
    Json(payload): Json<CreateLog>,
) -> impl IntoResponse {
    let mut logs = state.logs.write().await;

    let entry = LogEntry {
        id: Ulid::new().to_string(),
        message: payload.message,
        timestamp: Utc::now().to_rfc3339(),
    };

    logs.push(entry.clone());

    (StatusCode::CREATED, Json(entry))
}

async fn list_logs(State(state): State<AppState>) -> impl IntoResponse {
    let logs = state.logs.read().await;
    let list = logs.clone();
    Json(list)
}

async fn ping() -> impl IntoResponse {
    let resp = LogEntry {
        id: Ulid::new().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        message: String::from("Ping response from server!"),
    };

    (StatusCode::OK, Json(resp))
}
