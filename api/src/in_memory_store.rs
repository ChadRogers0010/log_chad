use common::LogEntry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryStore {
    logs: Arc<RwLock<Vec<LogEntry>>>,
}

#[axum::async_trait]
impl crate::LogStore for InMemoryStore {
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
