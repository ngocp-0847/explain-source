use crate::code_agent::{CodeAgent, CodeAnalysisRequest, CodeAnalysisResponse};
use crate::database::Database;
use crate::log_normalizer::LogNormalizer;
use crate::message_store::MsgStore;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

#[derive(Debug, thiserror::Error)]
pub enum GeminiAgentError {
    #[error("Process timeout after {0}s")]
    Timeout(u64),
    #[error("Process failed with exit code {0}")]
    ProcessFailed(i32),
    #[error("Executable not found: {0}")]
    ExecutableNotFound(String),
    #[error("Process spawn failed: {0}")]
    SpawnFailed(String),
    #[error("Working directory not accessible: {0}")]
    DirectoryNotAccessible(String),
    #[error("Authentication required: {0}")]
    AuthenticationRequired(String),
}

#[derive(Debug, Clone)]
pub struct GeminiAgentConfig {
    pub executable_path: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub working_dir: Option<String>,
    pub output_format: OutputFormat,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    StreamJson,
    StreamPartialOutput,
}

impl Default for GeminiAgentConfig {
    fn default() -> Self {
        Self {
            executable_path: "gemini".to_string(),
            timeout_seconds: 300, // 5 minutes
            max_retries: 2,
            working_dir: None,
            output_format: OutputFormat::StreamJson,
            api_key: std::env::var("GEMINI_API_KEY").ok(),
        }
    }
}

impl GeminiAgentConfig {
    pub fn from_env() -> Self {
        let output_format = match std::env::var("GEMINI_AGENT_OUTPUT_FORMAT")
            .unwrap_or_else(|_| "stream-json".to_string())
            .as_str()
        {
            "text" => OutputFormat::Text,
            "json" => OutputFormat::Json,
            "stream-json" => OutputFormat::StreamJson,
            "stream-partial" => OutputFormat::StreamPartialOutput,
            _ => OutputFormat::StreamJson,
        };

        Self {
            executable_path: std::env::var("GEMINI_AGENT_PATH")
                .unwrap_or_else(|_| "gemini".to_string()),
            timeout_seconds: std::env::var("GEMINI_AGENT_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
            max_retries: std::env::var("GEMINI_AGENT_MAX_RETRIES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2),
            working_dir: std::env::var("GEMINI_AGENT_WORKING_DIR").ok(),
            output_format,
            api_key: std::env::var("GEMINI_API_KEY").ok(),
        }
    }
}

#[derive(Debug)]
pub struct GeminiAgent {
    config: GeminiAgentConfig,
}

impl GeminiAgent {
    pub fn with_config(config: GeminiAgentConfig) -> Self {
        Self { config }
    }

    async fn execute_gemini_agent(
        &self,
        request: &CodeAnalysisRequest,
        working_directory: Option<String>,
        msg_store: &Arc<MsgStore>,
        normalizer: &LogNormalizer,
    ) -> Result<String> {
        info!("🎯 Executing Gemini analysis for: {}", request.code_context);
        
        // Validate working directory and code_context path
        let analysis_dir = working_directory.or(self.config.working_dir.clone());
        if let Some(ref dir) = analysis_dir {
            info!("📂 Analysis scope: {}", dir);
            // Validate directory exists and is accessible
            if let Err(e) = tokio::fs::metadata(dir).await {
                error!("⚠️ Không thể access directory {}: {}", dir, e);
                return Err(GeminiAgentError::DirectoryNotAccessible(dir.clone()).into());
            }
        }

        // Validate executable exists
        if self.config.executable_path.contains('/') || self.config.executable_path.contains('\\') {
            if let Err(_e) = tokio::fs::metadata(&self.config.executable_path).await {
                error!(
                    "⚠️ Gemini executable không tồn tại: {}",
                    self.config.executable_path
                );
                return Err(GeminiAgentError::ExecutableNotFound(
                    self.config.executable_path.clone(),
                )
                .into());
            }
        } else {
            debug!("Checking if '{}' exists in PATH", self.config.executable_path);
            if std::cfg!(unix) {
                if let Ok(output) = tokio::process::Command::new("which")
                    .arg(&self.config.executable_path)
                    .output()
                    .await
                {
                    if !output.status.success() {
                        error!(
                            "⚠️ Gemini CLI '{}' không tìm thấy trong PATH",
                            self.config.executable_path
                        );
                        error!("💡 Hãy install Gemini CLI: npm install -g @google/generative-ai-cli");
                        error!("💡 Hoặc set GEMINI_AGENT_PATH với absolute path đến executable");
                        return Err(GeminiAgentError::ExecutableNotFound(format!(
                            "'{}' not found in PATH",
                            self.config.executable_path
                        ))
                        .into());
                    }
                }
            }
        }

        // Execute with retry logic
        let mut last_error = None;
        for attempt in 1..=self.config.max_retries {
            info!(
                "🔄 Attempt {}/{} for Gemini analysis",
                attempt, self.config.max_retries
            );

            match self
                .spawn_gemini_process(request, analysis_dir.clone(), msg_store, normalizer)
                .await
            {
                Ok(result) => {
                    info!("✅ Gemini analysis completed successfully on attempt {}", attempt);
                    return Ok(result);
                }
                Err(e) => {
                    warn!("❌ Attempt {} failed: {}", attempt, e);
                    last_error = Some(e);

                    if attempt < self.config.max_retries {
                        info!("⏳ Waiting before retry...");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }

    async fn spawn_gemini_process(
        &self,
        request: &CodeAnalysisRequest,
        working_directory: Option<String>,
        msg_store: &Arc<MsgStore>,
        _normalizer: &LogNormalizer,
    ) -> Result<String> {
        let prompt = self.create_analysis_prompt(request);
        let ticket_id = request.ticket_id.clone();

        info!("🚀 Spawning Gemini CLI process: {}", self.config.executable_path);
        debug!("Prompt: {}", prompt);

        // Build Gemini CLI command
        // Format: gemini -p "prompt" (non-interactive mode)
        // Note: Gemini CLI does not support --output-format flag
        // Output will be parsed automatically based on actual format returned
        // Reference: https://github.com/google-gemini/gemini-cli
        let mut cmd = Command::new(&self.config.executable_path);

        // Add -p flag with prompt for non-interactive mode
        cmd.arg("-p").arg(&prompt);

        // Set working directory với absolute path đã được normalize
        if let Some(ref dir) = working_directory {
            info!("📂 Setting working directory cho Gemini CLI: {}", dir);
            cmd.current_dir(dir);
        } else {
            warn!("⚠️ Không có working directory, Gemini CLI sẽ chạy trong thư mục hiện tại của process");
        }

        // Set API key if available
        if let Some(ref api_key) = self.config.api_key {
            cmd.env("GEMINI_API_KEY", api_key);
        }

        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Spawn the process
        let mut child = cmd
            .spawn()
            .map_err(|e| GeminiAgentError::SpawnFailed(e.to_string()))?;

        // Close stdin immediately
        let _stdin = child.stdin.take();
        drop(_stdin);
        info!("🔒 Closed stdin to signal EOF to Gemini CLI");

        let stdout = child.stdout.take().ok_or_else(|| {
            GeminiAgentError::SpawnFailed("Failed to get stdout pipe".to_string())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            GeminiAgentError::SpawnFailed("Failed to get stderr pipe".to_string())
        })?;

        // Clone for async tasks
        let msg_store_clone = msg_store.clone();
        let ticket_id_clone = ticket_id.clone();

        // Spawn task to capture stdout and process JSON lines
        let stdout_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            let mut output_lines = Vec::new();
            let normalizer = LogNormalizer::new();

            // Buffer for merging delta messages from assistant
            let mut current_content = String::new();
            let mut last_timestamp: Option<String> = None;

            while let Ok(Some(line)) = lines.next_line().await {
                info!("📤 GEMINI STDOUT: {}", line);
                output_lines.push(line.clone());

                // Try to parse as JSON
                if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                    let msg_type = json_value.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    
                    // Handle assistant messages with delta
                    if msg_type == "message" {
                        if let Some(role_str) = json_value.get("role").and_then(|v| v.as_str()) {
                            if role_str == "assistant" {
                                // Extract content - can be string or already concatenated
                                let content_str = json_value.get("content")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .unwrap_or_default();

                                // Check if this is a delta message (delta: true)
                                if let Some(delta) = json_value.get("delta").and_then(|v| v.as_bool()) {
                                    if delta {
                                        // This is a delta message, accumulate content
                                        current_content.push_str(&content_str);
                                        
                                        // Store timestamp for final message
                                        if let Some(ts_str) = json_value.get("timestamp").and_then(|v| v.as_str()) {
                                            last_timestamp = Some(ts_str.to_string());
                                        }
                                        
                                        // Don't push individual delta messages
                                        continue;
                                    }
                                } else {
                                    // No delta field, could be final message or standalone
                                    // If we have buffered content, this might be a continuation or reset
                                    // For now, treat as final if there's no delta field and we have content
                                }
                                
                                // Final message (delta: false or no delta field), merge with buffer
                                if !current_content.is_empty() {
                                    // Merge buffered content with current content
                                    current_content.push_str(&content_str);
                                    
                                    // Create merged message JSON in unified format
                                    let merged_json = serde_json::json!({
                                        "type": "message",
                                        "role": "assistant",
                                        "content": current_content,
                                        "timestamp": last_timestamp.clone().unwrap_or_else(|| chrono::Utc::now().to_rfc3339())
                                    });
                                    
                                    let merged_line = serde_json::to_string(&merged_json)
                                        .unwrap_or_else(|_| line.clone());
                                    
                                    let entry = normalizer.normalize(merged_line, ticket_id_clone.clone());
                                    msg_store_clone.push(entry).await;
                                    
                                    // Reset buffer
                                    current_content.clear();
                                    last_timestamp = None;
                                } else if !content_str.is_empty() {
                                    // Standalone message without delta, process normally
                                    let entry = normalizer.normalize(line, ticket_id_clone.clone());
                                    msg_store_clone.push(entry).await;
                                }
                                continue;
                            }
                        }
                    }

                    // Not an assistant message, process normally
                    // Keep JSON string as-is for normalizer to parse
                    let entry = normalizer.normalize(line, ticket_id_clone.clone());
                    msg_store_clone.push(entry).await;
                } else {
                    // Not JSON, process as plain text
                    let entry = normalizer.normalize(line, ticket_id_clone.clone());
                    msg_store_clone.push(entry).await;
                }
            }

            // If there are remaining buffered delta messages, flush them
            if !current_content.is_empty() {
                let merged_json = serde_json::json!({
                    "type": "message",
                    "role": "assistant",
                    "content": current_content,
                    "timestamp": last_timestamp.unwrap_or_else(|| chrono::Utc::now().to_rfc3339())
                });
                let merged_line = serde_json::to_string(&merged_json).unwrap_or_default();
                let entry = normalizer.normalize(merged_line, ticket_id_clone.clone());
                msg_store_clone.push(entry).await;
            }

            info!(
                "📤 Finished reading Gemini stdout, total lines: {}",
                output_lines.len()
            );

            output_lines
        });

        // Spawn task to capture stderr
        let stderr_ticket_id = request.ticket_id.clone();
        let stderr_msg_store = msg_store.clone();

        let stderr_handle = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            let stderr_normalizer = LogNormalizer::new();
            let mut auth_error_detected = false;

            while let Ok(Some(line)) = lines.next_line().await {
                info!("⚠️ GEMINI STDERR: {}", line);

                // Check for authentication errors
                if line.contains("not logged in")
                    || line.contains("authentication")
                    || line.contains("login required")
                {
                    auth_error_detected = true;
                }

                let error_line = format!("ERROR: {}", line);
                let entry = stderr_normalizer.normalize(error_line, stderr_ticket_id.clone());
                stderr_msg_store.push(entry).await;
            }

            info!("⚠️ Finished reading Gemini stderr");
            auth_error_detected
        });

        // Wait for process to complete with timeout
        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
        info!(
            "⏳ Waiting for Gemini CLI process to complete (timeout: {}s)...",
            self.config.timeout_seconds
        );

        let process_result = timeout(timeout_duration, child.wait()).await;

        match process_result {
            Ok(Ok(status)) => {
                info!(
                    "✅ Gemini CLI process completed with exit code: {}",
                    status.code().unwrap_or(-1)
                );

                // Wait for log capture to complete
                let (stdout_result, stderr_result) = tokio::join!(stdout_handle, stderr_handle);

                let output_lines = stdout_result
                    .map_err(|e| GeminiAgentError::SpawnFailed(format!("Stdout task failed: {}", e)))?;

                let auth_error = stderr_result.unwrap_or(false);

                if !status.success() {
                    // Check if it's an authentication error
                    if auth_error {
                        return Err(GeminiAgentError::AuthenticationRequired(
                            "Gemini CLI chưa được đăng nhập. Hãy chạy 'gemini' và hoàn tất Google OAuth login.".to_string()
                        ).into());
                    }
                    return Err(GeminiAgentError::ProcessFailed(status.code().unwrap_or(-1)).into());
                }

                if output_lines.is_empty() {
                    warn!("⚠️ Gemini CLI produced no output");
                    return Ok("Analysis completed but no output generated".to_string());
                }

                Ok(output_lines.join("\n"))
            }
            Ok(Err(e)) => {
                error!("❌ Gemini process wait failed: {}", e);
                stdout_handle.abort();
                stderr_handle.abort();
                Err(GeminiAgentError::SpawnFailed(e.to_string()).into())
            }
            Err(_) => {
                error!(
                    "⏰ Gemini process timeout after {} seconds",
                    self.config.timeout_seconds
                );

                if let Err(e) = child.kill().await {
                    error!("Failed to kill timeout Gemini process: {}", e);
                }

                stdout_handle.abort();
                stderr_handle.abort();

                Err(GeminiAgentError::Timeout(self.config.timeout_seconds).into())
            }
        }
    }

    fn create_analysis_prompt(&self, request: &CodeAnalysisRequest) -> String {
        if request.code_context.is_empty() {
            format!(
                "Phân tích code để giúp QA hiểu business flow. Câu hỏi: {}",
                request.question
            )
        } else {
            format!(
                "Analyze the code in {} to help QA understand the business flow. Question: {}",
                request.code_context, request.question
            )
        }
    }
}

#[async_trait]
impl CodeAgent for GeminiAgent {
    async fn analyze_code(
        &self,
        request: CodeAnalysisRequest,
        msg_store: Arc<MsgStore>,
        database: Arc<Database>,
    ) -> Result<CodeAnalysisResponse> {
        info!("🚀 Bắt đầu phân tích code với Gemini cho ticket: {}", request.ticket_id);

        // Check if ticket exists, auto-create if not
        let ticket = database.get_ticket(&request.ticket_id).await?;
        if ticket.is_none() {
            info!(
                "🔧 Ticket {} chưa tồn tại, tự động tạo ticket",
                request.ticket_id
            );

            let auto_ticket = crate::database::TicketRecord {
                id: request.ticket_id.clone(),
                project_id: request.project_id.clone(),
                title: "Auto-created".to_string(),
                description: request.question.clone(),
                status: "in-progress".to_string(),
                code_context: Some(request.code_context.clone()),
                analysis_result: None,
                is_analyzing: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                mode: request.mode.clone(),
                plan_content: None,
                plan_created_at: None,
                required_approvals: 2,
            };

            database.create_ticket(&auto_ticket).await?;
            info!("✅ Đã tự động tạo ticket: {}", request.ticket_id);
        }

        // Create analysis session
        let session_id = database.create_session(&request.ticket_id).await?;

        // Update ticket status to analyzing
        database
            .update_ticket_analyzing(&request.ticket_id, true)
            .await?;

        let mut logs = Vec::new();
        let normalizer = LogNormalizer::new();

        // Send initial log
        let start_log = "🔄 Khởi động Gemini CLI...";
        let entry = normalizer.normalize(start_log.to_string(), request.ticket_id.clone());
        msg_store.push(entry).await;
        logs.push(start_log.to_string());

        // Get project directory for analysis scope
        let working_directory = if !request.project_id.is_empty() {
            if let Ok(Some(project)) = database.get_project(&request.project_id).await {
                info!("📂 Working directory: {}", project.directory_path);
                Some(project.directory_path)
            } else {
                error!("⚠️ Không tìm thấy project {}", request.project_id);
                None
            }
        } else {
            None
        };

        // Execute Gemini CLI analysis
        let result = match self
            .execute_gemini_agent(&request, working_directory, &msg_store, &normalizer)
            .await
        {
            Ok(output) => {
                info!("✅ Gemini CLI hoàn thành phân tích");

                // Send completion log
                let completion_log = "✅ Phân tích hoàn tất!";
                let mut entry =
                    normalizer.normalize(completion_log.to_string(), request.ticket_id.clone());
                entry.message_type = crate::message_store::LogMessageType::Result;
                msg_store.push(entry).await;
                logs.push(completion_log.to_string());

                // Update database with success
                database.complete_session(&session_id, "Success").await?;
                database
                    .update_ticket_result(&request.ticket_id, &output)
                    .await?;

                output
            }
            Err(e) => {
                error!("❌ Lỗi khi thực thi Gemini CLI: {}", e);

                // Send error log
                let error_log = format!("❌ Lỗi: {}", e);
                let entry = normalizer.normalize(error_log.clone(), request.ticket_id.clone());
                msg_store.push(entry).await;
                logs.push(error_log);

                // Update database with failure
                database.fail_session(&session_id, &e.to_string()).await?;
                database
                    .update_ticket_analyzing(&request.ticket_id, false)
                    .await?;

                format!("Không thể phân tích code do lỗi: {}", e)
            }
        };

        Ok(CodeAnalysisResponse {
            ticket_id: request.ticket_id,
            result,
            logs,
            success: true,
        })
    }
}
