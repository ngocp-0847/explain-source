use crate::database::Database;
use crate::message_store::MsgStore;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Request for code analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisRequest {
    pub ticket_id: String,
    pub code_context: String,
    pub question: String,
    pub project_id: String,
    pub mode: String, // "plan", "ask", or "edit"
}

/// Response from code analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeAnalysisResponse {
    pub ticket_id: String,
    pub result: String,
    pub logs: Vec<String>,
    pub success: bool,
}

/// Trait for code analysis agents
///
/// Implementations must be Send + Sync to work with Arc<dyn CodeAgent>
#[async_trait]
pub trait CodeAgent: Send + Sync {
    /// Analyze code based on the provided request
    ///
    /// # Arguments
    /// * `request` - The analysis request containing ticket info and question
    /// * `msg_store` - Message store for real-time log streaming
    /// * `database` - Database for persisting analysis results
    ///
    /// # Returns
    /// Result containing the analysis response or an error
    async fn analyze_code(
        &self,
        request: CodeAnalysisRequest,
        msg_store: Arc<MsgStore>,
        database: Arc<Database>,
    ) -> Result<CodeAnalysisResponse>;
}
