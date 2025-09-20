use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum::{async_trait, extract::State};
use chrono::{DateTime, Utc};
use common::{LogEntry, LogQuery};
use config::Config;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber;
use ulid::Ulid;

mod config;
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
    async fn count(&self) -> usize {
        self.logs.read().await.len()
    }
}

#[allow(unused)]
#[derive(Clone)]
struct AppState<DB: LogStore> {
    pub db: DB,
    pub cfg: config::Config,
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
#[tokio::main]
async fn main() {
    let cfg = Config::from_env().expect("Failed to load configuration");
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

#[async_trait]
trait LogStore: Clone + Send + Sync + 'static {
    async fn list_logs(&self) -> Vec<LogEntry>;
    async fn add_log(&self, entry: String) -> LogEntry;
    async fn count(&self) -> usize;
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
    if let Some(after_utc) = params.after.as_deref().and_then(parse_utc) {
        logs.retain(|log| matches_after(log, after_utc));
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

fn parse_utc(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn matches_after(log: &LogEntry, after: DateTime<Utc>) -> bool {
    parse_utc(&log.timestamp)
        .map(|log_dt| log_dt > after)
        .unwrap_or(false)
}

async fn count_logs<DB: LogStore>(State(state): State<AppState<DB>>) -> impl IntoResponse {
    let count = state.db.count().await;

    let count_response = LogEntry::new(format!("Count: {count}"));

    Json(count_response)
}

async fn ping() -> impl IntoResponse {
    let resp = LogEntry {
        id: Ulid::new().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        message: String::from("Ping response from server!"),
    };

    (StatusCode::OK, Json(resp))
}
