use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum::{async_trait, extract::State};
use std::net::SocketAddr;
use std::sync::Arc;

use common::{LogEntry, LogQuery};
use in_memory_store::*;

mod config;
mod in_memory_store;
mod pg_store;
mod tests;
mod utils;

pub use pg_store::*;

#[tokio::main]
async fn main() {
    let cfg = config::Config::from_env().expect("Failed to load configuration");

    // set up logging
    tracing_subscriber::fmt()
        .with_env_filter(cfg.log_level.clone())
        .init();

    let db: Arc<dyn LogStore> = if let Some(url) = &cfg.database_url {
        tracing::info!("Using {url}");
        Arc::new(PgStore::new(url).await.unwrap())
    } else {
        tracing::info!("Using InMemoryStore db");
        Arc::new(InMemoryStore::new())
    };

    let state = AppState {
        db,
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
struct AppState {
    pub db: Arc<dyn LogStore>,
    pub cfg: config::Config,
}

#[async_trait]
trait LogStore: Send + Sync + 'static {
    async fn list_logs(&self) -> anyhow::Result<Vec<LogEntry>>;
    async fn add_log(&self, entry: String) -> anyhow::Result<LogEntry>;
    async fn count(&self) -> anyhow::Result<usize>;
}

fn app_builder(state: AppState) -> Router {
    Router::<AppState>::new()
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

async fn create_log(
    State(state): State<AppState>,
    Json(payload): Json<CreateLog>,
) -> impl IntoResponse {
    let entry = state.db.add_log(payload.message).await;

    tracing::info!("add_log: {entry:?}");

    match entry {
        Ok(ok) => (StatusCode::CREATED, Json(ok)),

        Err(err) => {
            tracing::error!("App error: {err}");
            (StatusCode::BAD_REQUEST, Json(LogEntry::new(0.to_string())))
        }
    }
}

async fn list_logs(
    State(state): State<AppState>,
    Query(params): Query<LogQuery>,
) -> impl IntoResponse {
    let mut logs = match state.db.list_logs().await {
        Ok(ok) => ok,
        Err(err) => {
            tracing::error!("App error: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(vec![LogEntry::new("0".to_string())]),
            );
        }
    };

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

    (StatusCode::OK, Json(paginated))
}

async fn count_logs(State(state): State<AppState>) -> impl IntoResponse {
    let count = match state.db.count().await {
        Ok(ok) => ok,
        Err(err) => {
            tracing::error!("App error: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LogEntry::new("Failed to count logs".to_string())),
            );
        }
    };

    let count_response = LogEntry::new(format!("Count: {count}"));

    (StatusCode::OK, Json(count_response))
}

async fn ping() -> impl IntoResponse {
    let resp = LogEntry::new(String::from("Ping response from server!"));

    (StatusCode::OK, Json(resp))
}
