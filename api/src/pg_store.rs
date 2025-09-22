use common::LogEntry;
use sqlx::{PgPool, types::Uuid};

use crate::LogStore;

#[derive(Clone, Debug)]
pub struct PgStore {
    pub pool: PgPool,
}

impl PgStore {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }
}

#[axum::async_trait]
impl LogStore for PgStore {
    async fn list_logs(&self) -> anyhow::Result<Vec<LogEntry>> {
        let rows =
            sqlx::query!(r#"SELECT id, timestamp, message FROM logs ORDER BY timestamp DESC"#)
                .fetch_all(&self.pool)
                .await?;

        Ok(rows
            .into_iter()
            .map(|row| LogEntry {
                id: row.id.to_string(),
                timestamp: row.timestamp.to_rfc3339(),
                message: row.message,
            })
            .collect())
    }

    async fn add_log(&self, message: String) -> anyhow::Result<LogEntry> {
        let id = Uuid::new_v4();
        let timestamp = chrono::Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO logs (id, timestamp, message)
            VALUES ($1, $2, $3)
            "#,
            id,
            timestamp,
            message
        )
        .execute(&self.pool)
        .await?;

        Ok(LogEntry {
            id: id.to_string(),
            timestamp: timestamp.to_rfc3339(),
            message,
        })
    }

    async fn count(&self) -> anyhow::Result<usize> {
        let row = sqlx::query!("SELECT COUNT(*) as count FROM logs")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.count.unwrap_or(0) as usize)
    }
}
