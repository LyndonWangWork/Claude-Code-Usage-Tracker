//! SQLite storage for telemetry data

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use thiserror::Error;

use super::models::{ParsedEvent, ParsedMetric};

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Lock error")]
    Lock,
}

/// SQLite storage for telemetry data
pub struct TelemetryStorage {
    conn: Arc<Mutex<Connection>>,
}

impl TelemetryStorage {
    /// Create a new storage instance
    pub fn new(data_dir: Option<&str>) -> Result<Self, StorageError> {
        let db_path = Self::get_db_path(data_dir);

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;

        // Enable WAL mode for better concurrent access
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        storage.init_schema()?;

        Ok(storage)
    }

    /// Get the database file path
    fn get_db_path(data_dir: Option<&str>) -> PathBuf {
        if let Some(dir) = data_dir {
            PathBuf::from(dir).join("telemetry.db")
        } else {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("claude-code-usage-tracker")
                .join("telemetry.db")
        }
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;

        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                timestamp_ns INTEGER NOT NULL,
                value REAL NOT NULL,
                attributes TEXT NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            );

            CREATE INDEX IF NOT EXISTS idx_metrics_name ON metrics(name);
            CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON metrics(timestamp_ns);
            CREATE INDEX IF NOT EXISTS idx_metrics_name_timestamp ON metrics(name, timestamp_ns);

            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                timestamp_ns INTEGER NOT NULL,
                attributes TEXT NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            );

            CREATE INDEX IF NOT EXISTS idx_events_name ON events(name);
            CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp_ns);
            CREATE INDEX IF NOT EXISTS idx_events_name_timestamp ON events(name, timestamp_ns);
        "#)?;

        Ok(())
    }

    /// Store a batch of metrics
    pub fn store_metrics(&self, metrics: &[ParsedMetric]) -> Result<usize, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;
        let mut count = 0;

        for metric in metrics {
            let attributes_json = serde_json::to_string(&metric.attributes).unwrap_or_default();
            conn.execute(
                "INSERT INTO metrics (name, timestamp_ns, value, attributes) VALUES (?1, ?2, ?3, ?4)",
                params![metric.name, metric.timestamp_ns, metric.value, attributes_json],
            )?;
            count += 1;
        }

        Ok(count)
    }

    /// Store a batch of events
    pub fn store_events(&self, events: &[ParsedEvent]) -> Result<usize, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;
        let mut count = 0;

        for event in events {
            let attributes_json = serde_json::to_string(&event.attributes).unwrap_or_default();
            conn.execute(
                "INSERT INTO events (name, timestamp_ns, attributes) VALUES (?1, ?2, ?3)",
                params![event.name, event.timestamp_ns, attributes_json],
            )?;
            count += 1;
        }

        Ok(count)
    }

    /// Query metrics by name and time range
    pub fn query_metrics(
        &self,
        name: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<ParsedMetric>, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;

        let start_ns = start_time.map(|t| t.timestamp_nanos_opt().unwrap_or(0)).unwrap_or(0);
        let end_ns = end_time.map(|t| t.timestamp_nanos_opt().unwrap_or(i64::MAX)).unwrap_or(i64::MAX);

        let mut stmt = conn.prepare(
            "SELECT name, timestamp_ns, value, attributes FROM metrics
             WHERE name = ?1 AND timestamp_ns >= ?2 AND timestamp_ns <= ?3
             ORDER BY timestamp_ns ASC"
        )?;

        let rows = stmt.query_map(params![name, start_ns, end_ns], |row| {
            let name: String = row.get(0)?;
            let timestamp_ns: i64 = row.get(1)?;
            let value: f64 = row.get(2)?;
            let attributes_json: String = row.get(3)?;
            let attributes: std::collections::HashMap<String, String> =
                serde_json::from_str(&attributes_json).unwrap_or_default();

            Ok(ParsedMetric {
                name,
                timestamp_ns,
                value,
                attributes,
            })
        })?;

        let mut metrics = Vec::new();
        for row in rows {
            metrics.push(row?);
        }

        Ok(metrics)
    }

    /// Query all metrics matching a prefix (e.g., "claude_code.")
    pub fn query_metrics_by_prefix(
        &self,
        prefix: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<ParsedMetric>, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;

        let start_ns = start_time.map(|t| t.timestamp_nanos_opt().unwrap_or(0)).unwrap_or(0);
        let end_ns = end_time.map(|t| t.timestamp_nanos_opt().unwrap_or(i64::MAX)).unwrap_or(i64::MAX);
        let prefix_pattern = format!("{}%", prefix);

        let mut stmt = conn.prepare(
            "SELECT name, timestamp_ns, value, attributes FROM metrics
             WHERE name LIKE ?1 AND timestamp_ns >= ?2 AND timestamp_ns <= ?3
             ORDER BY timestamp_ns ASC"
        )?;

        let rows = stmt.query_map(params![prefix_pattern, start_ns, end_ns], |row| {
            let name: String = row.get(0)?;
            let timestamp_ns: i64 = row.get(1)?;
            let value: f64 = row.get(2)?;
            let attributes_json: String = row.get(3)?;
            let attributes: std::collections::HashMap<String, String> =
                serde_json::from_str(&attributes_json).unwrap_or_default();

            Ok(ParsedMetric {
                name,
                timestamp_ns,
                value,
                attributes,
            })
        })?;

        let mut metrics = Vec::new();
        for row in rows {
            metrics.push(row?);
        }

        Ok(metrics)
    }

    /// Query events by name and time range
    pub fn query_events(
        &self,
        name: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<ParsedEvent>, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;

        let start_ns = start_time.map(|t| t.timestamp_nanos_opt().unwrap_or(0)).unwrap_or(0);
        let end_ns = end_time.map(|t| t.timestamp_nanos_opt().unwrap_or(i64::MAX)).unwrap_or(i64::MAX);

        let mut stmt = conn.prepare(
            "SELECT name, timestamp_ns, attributes FROM events
             WHERE name = ?1 AND timestamp_ns >= ?2 AND timestamp_ns <= ?3
             ORDER BY timestamp_ns ASC"
        )?;

        let rows = stmt.query_map(params![name, start_ns, end_ns], |row| {
            let name: String = row.get(0)?;
            let timestamp_ns: i64 = row.get(1)?;
            let attributes_json: String = row.get(2)?;
            let attributes: std::collections::HashMap<String, String> =
                serde_json::from_str(&attributes_json).unwrap_or_default();

            Ok(ParsedEvent {
                name,
                timestamp_ns,
                attributes,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }

        Ok(events)
    }

    /// Query all events matching a prefix
    pub fn query_events_by_prefix(
        &self,
        prefix: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<ParsedEvent>, StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;

        let start_ns = start_time.map(|t| t.timestamp_nanos_opt().unwrap_or(0)).unwrap_or(0);
        let end_ns = end_time.map(|t| t.timestamp_nanos_opt().unwrap_or(i64::MAX)).unwrap_or(i64::MAX);
        let prefix_pattern = format!("{}%", prefix);

        let mut stmt = conn.prepare(
            "SELECT name, timestamp_ns, attributes FROM events
             WHERE name LIKE ?1 AND timestamp_ns >= ?2 AND timestamp_ns <= ?3
             ORDER BY timestamp_ns ASC"
        )?;

        let rows = stmt.query_map(params![prefix_pattern, start_ns, end_ns], |row| {
            let name: String = row.get(0)?;
            let timestamp_ns: i64 = row.get(1)?;
            let attributes_json: String = row.get(2)?;
            let attributes: std::collections::HashMap<String, String> =
                serde_json::from_str(&attributes_json).unwrap_or_default();

            Ok(ParsedEvent {
                name,
                timestamp_ns,
                attributes,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }

        Ok(events)
    }

    /// Delete old data beyond retention period (default 90 days)
    pub fn cleanup_old_data(&self, retention_days: u32) -> Result<(usize, usize), StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;

        let cutoff_ns = Utc::now()
            .checked_sub_signed(chrono::Duration::days(retention_days as i64))
            .map(|t| t.timestamp_nanos_opt().unwrap_or(0))
            .unwrap_or(0);

        let metrics_deleted = conn.execute(
            "DELETE FROM metrics WHERE timestamp_ns < ?1",
            params![cutoff_ns],
        )?;

        let events_deleted = conn.execute(
            "DELETE FROM events WHERE timestamp_ns < ?1",
            params![cutoff_ns],
        )?;

        Ok((metrics_deleted, events_deleted))
    }

    /// Get total counts for diagnostics
    pub fn get_counts(&self) -> Result<(i64, i64), StorageError> {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;

        let metrics_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM metrics",
            [],
            |row| row.get(0),
        )?;

        let events_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM events",
            [],
            |row| row.get(0),
        )?;

        Ok((metrics_count, events_count))
    }
}

impl Clone for TelemetryStorage {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}
