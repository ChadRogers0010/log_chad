use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum::{async_trait, extract::State};
use std::net::SocketAddr;

use common::{LogEntry, LogQuery};
use in_memory_store::*;

mod config;
mod in_memory_store;
mod tests;
mod utils;

#[tokio::main]
async fn main() {
    let cfg = config::Config::from_env().expect("Failed to load configuration");
    // set up logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    let state = AppState {
        db: InMemoryStore::default(),
        cfg: cfg.clone(),
    };
    let app = app_builder(state);

    let addr = SocketAddr::from((cfg.address, cfg.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Bind failed: {addr}");

    tracing::info!("Server running on http://{addr}");

    axum::serve::serve(listener, app).await.unwrap();
}

#[allow(unused)]
#[derive(Clone)]
struct AppState<DB: LogStore> {
    pub db: DB,
    pub cfg: config::Config,
}

#[async_trait]
trait LogStore: Clone + Send + Sync + 'static {
    async fn list_logs(&self) -> Vec<LogEntry>;
    async fn add_log(&self, entry: String) -> LogEntry;
    async fn count(&self) -> usize;
}

fn app_builder<DB: LogStore>(state: AppState<DB>) -> Router {
    Router::<AppState<DB>>::new()
        .route("/", get(root))
        .route("/ping", get(ping))
        .route("/logs", post(create_log))
        .route("/logs", get(list_logs))
        .route("/logs/count", get(count_logs))
        .with_state(state)
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

async fn list_logs<DB: LogStore>(
    State(state): State<AppState<DB>>,
    Query(params): Query<LogQuery>,
) -> impl IntoResponse {
    let mut logs = state.db.list_logs().await;

    // Filter by time after
    if let Some(after_utc) = params.after.as_deref().and_then(utils::parse_utc) {
        logs.retain(|log| utils::matches_after(log, after_utc));
    }

    // Filter by contains
    if let Some(substr) = &params.contains {
        logs.retain(|log| log.message.contains(substr));
    }

    // Sort and paginate
    logs.sort_by_key(|log| log.timestamp.clone());

    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(50);
    let paginated = logs
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<LogEntry>>();

    Json(paginated)
}

async fn count_logs<DB: LogStore>(State(state): State<AppState<DB>>) -> impl IntoResponse {
    let count = state.db.count().await;

    let count_response = LogEntry::new(format!("Count: {count}"));

    Json(count_response)
}

async fn ping() -> impl IntoResponse {
    let resp = LogEntry::new(String::from("Ping response from server!"));

    (StatusCode::OK, Json(resp))
}
