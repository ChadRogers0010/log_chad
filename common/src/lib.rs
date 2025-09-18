use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LogEntry {
    pub id: i64,
    pub timestamp: u64,
    pub level: String,
    pub source: String,
    pub message: String,
}
