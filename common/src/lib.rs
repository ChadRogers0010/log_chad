use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: String,
    pub message: String,
}

impl LogEntry {
    pub fn new(message: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            message,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogQuery {
    pub after: Option<String>,
    pub contains: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
