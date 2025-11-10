use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, FromRow, Row};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub directory_path: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TicketRecord {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub code_context: Option<String>,
    pub analysis_result: Option<String>,
    pub is_analyzing: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredLogRecord {
    pub id: String,
    pub ticket_id: String,
    pub message_type: String,
    pub content: String,
    pub raw_log: Option<String>,
    pub metadata: Option<String>, // JSON string
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisSession {
    pub id: String,
    pub ticket_id: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Debug)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn init_schema(&self) -> Result<()> {
        // Create projects table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                directory_path TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create tickets table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tickets (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL CHECK(status IN ('todo', 'in-progress', 'done')),
                code_context TEXT,
                analysis_result TEXT,
                is_analyzing BOOLEAN DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Add project_id column to existing tickets table if it doesn't exist
        let _ = sqlx::query(
            r#"
            ALTER TABLE tickets ADD COLUMN project_id TEXT
            "#
        )
        .execute(&self.pool)
        .await;

        // Create index for tickets by project
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tickets_project_id ON tickets(project_id)")
            .execute(&self.pool)
            .await?;

        // Create structured_logs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS structured_logs (
                id TEXT PRIMARY KEY,
                ticket_id TEXT NOT NULL,
                message_type TEXT NOT NULL CHECK(message_type IN ('tool_use', 'assistant', 'error', 'system', 'result')),
                content TEXT NOT NULL,
                raw_log TEXT,
                metadata TEXT,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_logs_ticket_id ON structured_logs(ticket_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON structured_logs(timestamp)")
            .execute(&self.pool)
            .await?;

        // Create analysis_sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS analysis_sessions (
                id TEXT PRIMARY KEY,
                ticket_id TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                status TEXT NOT NULL CHECK(status IN ('running', 'completed', 'failed', 'cancelled')),
                error_message TEXT,
                FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Clear all existing data (for migration)
    pub async fn clear_all_tickets(&self) -> Result<()> {
        sqlx::query("DELETE FROM analysis_sessions")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("DELETE FROM structured_logs")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("DELETE FROM tickets")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("DELETE FROM projects")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Project CRUD operations
    pub async fn create_project(&self, project: &ProjectRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO projects (id, name, description, directory_path, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(&project.id)
        .bind(&project.name)
        .bind(&project.description)
        .bind(&project.directory_path)
        .bind(&project.created_at)
        .bind(&project.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_project(&self, id: &str) -> Result<Option<ProjectRecord>> {
        let project = sqlx::query_as::<_, ProjectRecord>(
            "SELECT * FROM projects WHERE id = ?1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(project)
    }

    pub async fn list_projects(&self) -> Result<Vec<ProjectRecord>> {
        let projects = sqlx::query_as::<_, ProjectRecord>(
            "SELECT * FROM projects ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(projects)
    }

    pub async fn update_project(&self, project: &ProjectRecord) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE projects
            SET name = ?1, description = ?2, directory_path = ?3, updated_at = ?4
            WHERE id = ?5
            "#,
        )
        .bind(&project.name)
        .bind(&project.description)
        .bind(&project.directory_path)
        .bind(&project.updated_at)
        .bind(&project.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_project(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM projects WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Ticket CRUD operations
    pub async fn create_ticket(&self, ticket: &TicketRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO tickets (id, project_id, title, description, status, code_context, analysis_result, is_analyzing, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
        )
        .bind(&ticket.id)
        .bind(&ticket.project_id)
        .bind(&ticket.title)
        .bind(&ticket.description)
        .bind(&ticket.status)
        .bind(&ticket.code_context)
        .bind(&ticket.analysis_result)
        .bind(ticket.is_analyzing)
        .bind(&ticket.created_at)
        .bind(&ticket.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_ticket(&self, ticket: &TicketRecord) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE tickets
            SET project_id = ?1, title = ?2, description = ?3, status = ?4, code_context = ?5,
                analysis_result = ?6, is_analyzing = ?7, updated_at = ?8
            WHERE id = ?9
            "#,
        )
        .bind(&ticket.project_id)
        .bind(&ticket.title)
        .bind(&ticket.description)
        .bind(&ticket.status)
        .bind(&ticket.code_context)
        .bind(&ticket.analysis_result)
        .bind(ticket.is_analyzing)
        .bind(&ticket.updated_at)
        .bind(&ticket.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_ticket_status(&self, ticket_id: &str, status: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE tickets
            SET status = ?1, updated_at = ?2
            WHERE id = ?3
            "#,
        )
        .bind(status)
        .bind(now)
        .bind(ticket_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_ticket_analyzing(&self, ticket_id: &str, is_analyzing: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE tickets
            SET is_analyzing = ?1, updated_at = ?2
            WHERE id = ?3
            "#,
        )
        .bind(is_analyzing)
        .bind(now)
        .bind(ticket_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_ticket_result(&self, ticket_id: &str, result: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE tickets
            SET analysis_result = ?1, is_analyzing = ?2, updated_at = ?3
            WHERE id = ?4
            "#,
        )
        .bind(result)
        .bind(false)
        .bind(now)
        .bind(ticket_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_ticket(&self, id: &str) -> Result<Option<TicketRecord>> {
        let ticket = sqlx::query_as::<_, TicketRecord>(
            "SELECT * FROM tickets WHERE id = ?1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(ticket)
    }

    pub async fn list_tickets(&self) -> Result<Vec<TicketRecord>> {
        let tickets = sqlx::query_as::<_, TicketRecord>(
            "SELECT * FROM tickets ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tickets)
    }

    pub async fn list_tickets_by_project(&self, project_id: &str) -> Result<Vec<TicketRecord>> {
        let tickets = sqlx::query_as::<_, TicketRecord>(
            "SELECT * FROM tickets WHERE project_id = ?1 ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tickets)
    }

    pub async fn delete_ticket(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM tickets WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Log operations
    pub async fn save_log(&self, log: &StructuredLogRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO structured_logs (id, ticket_id, message_type, content, raw_log, metadata, timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(&log.id)
        .bind(&log.ticket_id)
        .bind(&log.message_type)
        .bind(&log.content)
        .bind(&log.raw_log)
        .bind(&log.metadata)
        .bind(&log.timestamp)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_logs_batch(&self, logs: &[StructuredLogRecord]) -> Result<()> {
        if logs.is_empty() {
            return Ok(());
        }

        // Use a transaction for batch insert
        let mut tx = self.pool.begin().await?;

        for log in logs {
            sqlx::query(
                r#"
                INSERT INTO structured_logs (id, ticket_id, message_type, content, raw_log, metadata, timestamp)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
            )
            .bind(&log.id)
            .bind(&log.ticket_id)
            .bind(&log.message_type)
            .bind(&log.content)
            .bind(&log.raw_log)
            .bind(&log.metadata)
            .bind(&log.timestamp)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn count_logs_for_ticket(&self, ticket_id: &str) -> Result<u64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM structured_logs WHERE ticket_id = ?1"
        )
        .bind(ticket_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count as u64)
    }

    pub async fn get_logs_for_ticket(
        &self,
        ticket_id: &str,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<StructuredLogRecord>> {
        // Ensure limit is always valid: minimum 1, maximum 1000, default 100
        let limit = limit.unwrap_or(100).clamp(1, 1000);
        let offset = offset.unwrap_or(0);

        tracing::debug!(
            "get_logs_for_ticket: ticket_id={}, limit={}, offset={}",
            ticket_id,
            limit,
            offset
        );

        let logs = sqlx::query(
            "SELECT id, ticket_id, message_type, content, raw_log, metadata, timestamp 
             FROM structured_logs 
             WHERE ticket_id = ?1 
             ORDER BY timestamp ASC 
             LIMIT ?2 OFFSET ?3"
        )
        .bind(ticket_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in logs {
            result.push(StructuredLogRecord {
                id: row.get("id"),
                ticket_id: row.get("ticket_id"),
                message_type: row.get("message_type"),
                content: row.get("content"),
                raw_log: row.get("raw_log"),
                metadata: row.get("metadata"),
                timestamp: row.get("timestamp"),
            });
        }

        Ok(result)
    }

    pub async fn clear_logs_for_ticket(&self, ticket_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM structured_logs WHERE ticket_id = ?1")
            .bind(ticket_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Analysis session operations
    pub async fn create_session(&self, ticket_id: &str) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let started_at = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO analysis_sessions (id, ticket_id, started_at, status)
            VALUES (?1, ?2, ?3, 'running')
            "#,
        )
        .bind(&session_id)
        .bind(ticket_id)
        .bind(started_at)
        .execute(&self.pool)
        .await?;

        Ok(session_id)
    }

    pub async fn complete_session(&self, session_id: &str, _result: &str) -> Result<()> {
        let completed_at = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE analysis_sessions
            SET status = 'completed', completed_at = ?1
            WHERE id = ?2
            "#,
        )
        .bind(completed_at)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn fail_session(&self, session_id: &str, error: &str) -> Result<()> {
        let completed_at = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE analysis_sessions
            SET status = 'failed', completed_at = ?1, error_message = ?2
            WHERE id = ?3
            "#,
        )
        .bind(completed_at)
        .bind(error)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn cancel_session(&self, session_id: &str, reason: &str) -> Result<()> {
        let completed_at = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE analysis_sessions
            SET status = 'cancelled', completed_at = ?1, error_message = ?2
            WHERE id = ?3
            "#,
        )
        .bind(completed_at)
        .bind(reason)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_active_session_by_ticket(&self, ticket_id: &str) -> Result<Option<AnalysisSession>> {
        let session = sqlx::query_as::<_, AnalysisSession>(
            "SELECT * FROM analysis_sessions 
             WHERE ticket_id = ?1 AND status = 'running' 
             ORDER BY started_at DESC LIMIT 1"
        )
        .bind(ticket_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn run_migrations(&self) -> Result<()> {
        // Check migrations table exists
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS migrations (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL
            )"
        )
        .execute(&self.pool)
        .await?;

        // Run 001_add_result_message_type if not applied
        let migration_name = "001_add_result_message_type";
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM migrations WHERE name = ?1"
        )
        .bind(migration_name)
        .fetch_one(&self.pool)
        .await?;

        if exists == 0 {
            // Read migration SQL file
            let migration_sql = include_str!("../migrations/001_add_result_message_type.sql");
            
            // Execute migration SQL
            sqlx::query(migration_sql)
                .execute(&self.pool)
                .await?;
            
            // Mark as applied
            sqlx::query("INSERT INTO migrations (name, applied_at) VALUES (?1, ?2)")
                .bind(migration_name)
                .bind(chrono::Utc::now().to_rfc3339())
                .execute(&self.pool)
                .await?;
        }

        // Run 002_add_cancelled_status if not applied
        let migration_name_002 = "002_add_cancelled_status";
        let exists_002 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM migrations WHERE name = ?1"
        )
        .bind(migration_name_002)
        .fetch_one(&self.pool)
        .await?;

        if exists_002 == 0 {
            // Read migration SQL file
            let migration_sql = include_str!("../migrations/002_add_cancelled_status.sql");
            
            // Execute migration SQL
            sqlx::query(migration_sql)
                .execute(&self.pool)
                .await?;
            
            // Mark as applied
            sqlx::query("INSERT INTO migrations (name, applied_at) VALUES (?1, ?2)")
                .bind(migration_name_002)
                .bind(chrono::Utc::now().to_rfc3339())
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }
}
