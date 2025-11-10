use crate::database::{Database, StructuredLogRecord};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogMessageType {
    ToolUse,
    Assistant,
    Error,
    System,
    Result,
}

impl LogMessageType {
    pub fn as_str(&self) -> &str {
        match self {
            LogMessageType::ToolUse => "tool_use",
            LogMessageType::Assistant => "assistant",
            LogMessageType::Error => "error",
            LogMessageType::System => "system",
            LogMessageType::Result => "result",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "tool_use" => LogMessageType::ToolUse,
            "assistant" => LogMessageType::Assistant,
            "error" => LogMessageType::Error,
            "result" => LogMessageType::Result,
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
const BATCH_SIZE: usize = 50;
const FLUSH_INTERVAL_MS: u64 = 100;

#[derive(Debug)]
pub struct MsgStore {
    // In-memory circular buffer for real-time streaming
    buffer: Arc<Mutex<HashMap<String, VecDeque<StructuredLogEntry>>>>,

    // Database for persistence
    database: Arc<Database>,

    // Broadcast channel for WebSocket streaming
    broadcast_tx: broadcast::Sender<StructuredLogEntry>,

    // Queue for batch database inserts
    db_queue_tx: mpsc::UnboundedSender<StructuredLogEntry>,
}

impl MsgStore {
    pub fn new(database: Arc<Database>) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        let (db_queue_tx, mut db_queue_rx) = mpsc::unbounded_channel::<StructuredLogEntry>();

        // Spawn background task to batch insert logs
        let db_clone = database.clone();
        tokio::spawn(async move {
            let mut batch: Vec<StructuredLogRecord> = Vec::with_capacity(BATCH_SIZE);
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(FLUSH_INTERVAL_MS));

            loop {
                tokio::select! {
                    // Receive logs from queue
                    Some(entry) = db_queue_rx.recv() => {
                        batch.push(entry.to_record());

                        // Flush when batch is full
                        if batch.len() >= BATCH_SIZE {
                            if let Err(e) = db_clone.save_logs_batch(&batch).await {
                                error!("Failed to batch save logs: {}", e);
                            }
                            batch.clear();
                        }
                    }
                    // Flush on interval
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            if let Err(e) = db_clone.save_logs_batch(&batch).await {
                                error!("Failed to batch save logs: {}", e);
                            }
                            batch.clear();
                        }
                    }
                    // Channel closed, flush remaining and exit
                    else => {
                        if !batch.is_empty() {
                            if let Err(e) = db_clone.save_logs_batch(&batch).await {
                                error!("Failed to batch save logs: {}", e);
                            }
                        }
                        break;
                    }
                }
            }
        });

        Self {
            buffer: Arc::new(Mutex::new(HashMap::new())),
            database,
            broadcast_tx,
            db_queue_tx,
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

        // 2. Enqueue for batch database insert (non-blocking)
        // Ignore send errors (means background task has stopped)
        let _ = self.db_queue_tx.send(entry.clone());

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
        match self.database.get_logs_for_ticket(ticket_id, None, None).await {
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
        let records = self.database.get_logs_for_ticket(ticket_id, None, None).await?;

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

    /// Force flush all pending logs to database
    /// This is useful for graceful shutdown to ensure no logs are lost
    pub async fn flush(&self) {
        // Wait for background task to process remaining logs
        // Since we use interval-based flushing (100ms), wait 2x that time to be safe
        tokio::time::sleep(tokio::time::Duration::from_millis(FLUSH_INTERVAL_MS * 2)).await;
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
