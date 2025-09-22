use common::LogEntry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, Default)]
pub struct InMemoryStore {
    logs: Arc<RwLock<Vec<LogEntry>>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[axum::async_trait]
impl crate::LogStore for InMemoryStore {
    async fn list_logs(&self) -> anyhow::Result<Vec<LogEntry>> {
        Ok(self.logs.read().await.clone())
    }

    async fn add_log(&self, message: String) -> anyhow::Result<LogEntry> {
        let entry = LogEntry::new(message);
        self.logs.write().await.push(entry.clone());
        Ok(entry)
    }

    async fn count(&self) -> anyhow::Result<usize> {
        Ok(self.logs.read().await.len())
    }
}
