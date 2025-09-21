use chrono::{DateTime, Utc};
use common::LogEntry;

pub fn parse_utc(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

pub fn matches_after(log: &LogEntry, after: DateTime<Utc>) -> bool {
    parse_utc(&log.timestamp)
        .map(|log_dt| log_dt > after)
        .unwrap_or(false)
}
