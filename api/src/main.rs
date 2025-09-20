use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum::{async_trait, extract::State};
use chrono::Utc;
use common::LogEntry;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber;
use ulid::Ulid;

mod tests;

#[derive(Clone, Default)]
struct InMemoryStore {
    logs: Arc<RwLock<Vec<LogEntry>>>,
}
#[async_trait]
impl LogStore for InMemoryStore {
    async fn list_logs(&self) -> Vec<LogEntry> {
        self.logs.read().await.clone()
    }

    async fn add_log(&self, message: String) -> LogEntry {
        let entry = LogEntry::new(message);
        self.logs.write().await.push(entry.clone());
        entry
    }
}

#[derive(Clone)]
struct AppState<DB: LogStore> {
    pub db: DB,
}

fn app_builder<DB: LogStore>(db: DB) -> Router {
    let state = AppState { db };

    Router::<AppState<DB>>::new()
        .route("/", get(root))
        .route("/ping", get(ping))
        .route("/logs", post(create_log))
        .route("/logs", get(list_logs))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    // set up logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    let database = InMemoryStore::default();
    let app = app_builder(database);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Bind failed: {addr}");

    tracing::info!("Server running on http://{addr}");

    axum::serve::serve(listener, app).await.unwrap();
}

#[async_trait]
trait LogStore: Clone + Send + Sync + 'static {
    async fn list_logs(&self) -> Vec<LogEntry>;
    async fn add_log(&self, entry: String) -> LogEntry;
}

async fn root() -> &'static str {
    "Hello from Chad Log API!"
}

#[derive(Debug, serde::Deserialize)]
struct CreateLog {
    message: String,
}

async fn create_log<DB: LogStore>(
    State(state): State<AppState<DB>>,
    Json(payload): Json<CreateLog>,
) -> impl IntoResponse {
    let entry = state.db.add_log(payload.message).await;

    (StatusCode::CREATED, Json(entry))
}

async fn list_logs<DB: LogStore>(State(state): State<AppState<DB>>) -> impl IntoResponse {
    let logs = state.db.list_logs().await;
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
