use crate::database::{Database, StructuredLogRecord};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogMessageType {
    ToolUse,
    Assistant,
    Error,
    System,
}

impl LogMessageType {
    pub fn as_str(&self) -> &str {
        match self {
            LogMessageType::ToolUse => "tool_use",
            LogMessageType::Assistant => "assistant",
            LogMessageType::Error => "error",
            LogMessageType::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "tool_use" => LogMessageType::ToolUse,
            "assistant" => LogMessageType::Assistant,
            "error" => LogMessageType::Error,
            _ => LogMessageType::System,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredLogEntry {
    pub id: String,
    pub ticket_id: String,
    pub message_type: LogMessageType,
    pub content: String,
    pub raw_log: Option<String>,
    pub metadata: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl StructuredLogEntry {
    pub fn to_record(&self) -> StructuredLogRecord {
        StructuredLogRecord {
            id: self.id.clone(),
            ticket_id: self.ticket_id.clone(),
            message_type: self.message_type.as_str().to_string(),
            content: self.content.clone(),
            raw_log: self.raw_log.clone(),
            metadata: if self.metadata.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&self.metadata).unwrap_or_default())
            },
            timestamp: self.timestamp.to_rfc3339(),
        }
    }

    pub fn from_record(record: StructuredLogRecord) -> Self {
        let metadata = if let Some(meta_str) = record.metadata {
            serde_json::from_str(&meta_str).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self {
            id: record.id,
            ticket_id: record.ticket_id,
            message_type: LogMessageType::from_str(&record.message_type),
            content: record.content,
            raw_log: record.raw_log,
            metadata,
            timestamp: chrono::DateTime::parse_from_rfc3339(&record.timestamp)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc),
        }
    }
}

const MAX_BUFFER_SIZE: usize = 1000;

#[derive(Debug)]
pub struct MsgStore {
    // In-memory circular buffer for real-time streaming
    buffer: Arc<Mutex<HashMap<String, VecDeque<StructuredLogEntry>>>>,

    // Database for persistence
    database: Arc<Database>,

    // Broadcast channel for WebSocket streaming
    broadcast_tx: broadcast::Sender<StructuredLogEntry>,
}

impl MsgStore {
    pub fn new(database: Arc<Database>) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);

        Self {
            buffer: Arc::new(Mutex::new(HashMap::new())),
            database,
            broadcast_tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<StructuredLogEntry> {
        self.broadcast_tx.subscribe()
    }

    pub async fn push(&self, entry: StructuredLogEntry) {
        // 1. Add to in-memory buffer with circular buffer behavior
        {
            let mut buffer = self.buffer.lock().await;
            let ticket_logs = buffer
                .entry(entry.ticket_id.clone())
                .or_insert_with(VecDeque::new);

            ticket_logs.push_back(entry.clone());

            // Keep buffer size limited (circular buffer)
            if ticket_logs.len() > MAX_BUFFER_SIZE {
                ticket_logs.pop_front();
            }
        }

        // 2. Persist to database asynchronously (fire and forget)
        let db = self.database.clone();
        let record = entry.to_record();
        tokio::spawn(async move {
            if let Err(e) = db.save_log(&record).await {
                error!("Failed to save log to database: {}", e);
            }
        });

        // 3. Broadcast to all WebSocket subscribers
        // Ignore send errors (means no active subscribers)
        let _ = self.broadcast_tx.send(entry);
    }

    pub async fn get_logs(&self, ticket_id: &str) -> Vec<StructuredLogEntry> {
        // Try in-memory buffer first (fast path)
        {
            let buffer = self.buffer.lock().await;
            if let Some(logs) = buffer.get(ticket_id) {
                return logs.iter().cloned().collect();
            }
        }

        // Fallback to database if not in memory
        match self.database.get_logs_for_ticket(ticket_id).await {
            Ok(records) => records
                .into_iter()
                .map(StructuredLogEntry::from_record)
                .collect(),
            Err(e) => {
                error!("Failed to load logs from database: {}", e);
                Vec::new()
            }
        }
    }

    pub async fn clear_logs(&self, ticket_id: &str) -> Result<()> {
        // Clear from in-memory buffer
        {
            let mut buffer = self.buffer.lock().await;
            buffer.remove(ticket_id);
        }

        // Clear from database
        self.database.clear_logs_for_ticket(ticket_id).await?;

        Ok(())
    }

    pub async fn get_buffer_stats(&self) -> HashMap<String, usize> {
        let buffer = self.buffer.lock().await;
        buffer
            .iter()
            .map(|(ticket_id, logs)| (ticket_id.clone(), logs.len()))
            .collect()
    }

    // Load logs from database into memory buffer (for server restart recovery)
    pub async fn warm_cache(&self, ticket_id: &str) -> Result<()> {
        let records = self.database.get_logs_for_ticket(ticket_id).await?;

        let mut buffer = self.buffer.lock().await;
        let ticket_logs = buffer.entry(ticket_id.to_string()).or_insert_with(VecDeque::new);

        ticket_logs.clear();
        for record in records {
            let entry = StructuredLogEntry::from_record(record);
            ticket_logs.push_back(entry);

            // Maintain size limit
            if ticket_logs.len() > MAX_BUFFER_SIZE {
                ticket_logs.pop_front();
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circular_buffer() {
        let db = Arc::new(Database::new("sqlite::memory:").await.unwrap());
        db.init_schema().await.unwrap();
        let store = MsgStore::new(db);

        // Push more than MAX_BUFFER_SIZE logs
        for i in 0..1500 {
            let entry = StructuredLogEntry {
                id: format!("log-{}", i),
                ticket_id: "test-ticket".to_string(),
                message_type: LogMessageType::System,
                content: format!("Log message {}", i),
                raw_log: None,
                metadata: HashMap::new(),
                timestamp: chrono::Utc::now(),
            };
            store.push(entry).await;
        }

        // Wait for async operations
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let logs = store.get_logs("test-ticket").await;

        // Buffer should be limited to MAX_BUFFER_SIZE
        assert!(logs.len() <= MAX_BUFFER_SIZE);
    }
}
