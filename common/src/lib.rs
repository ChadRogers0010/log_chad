use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: String,
    pub message: String,
}
