use crate::code_agent::{CodeAgent, CodeAnalysisRequest, CodeAnalysisResponse};
use crate::database::Database;
use crate::log_normalizer::LogNormalizer;
use crate::message_store::MsgStore;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

#[derive(Debug, thiserror::Error)]
pub enum CursorAgentError {
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
}

#[derive(Debug, Clone)]
pub struct CursorAgentConfig {
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

impl Default for CursorAgentConfig {
    fn default() -> Self {
        Self {
            executable_path: "cursor-agent".to_string(),
            timeout_seconds: 300, // 5 minutes
            max_retries: 2,
            working_dir: None,
            output_format: OutputFormat::StreamJson,
            api_key: std::env::var("CURSOR_API_KEY").ok(),
        }
    }
}

impl CursorAgentConfig {
    pub fn from_env() -> Self {
        let output_format = match std::env::var("CURSOR_AGENT_OUTPUT_FORMAT")
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
            executable_path: std::env::var("CURSOR_AGENT_PATH")
                .unwrap_or_else(|_| "cursor-agent".to_string()),
            timeout_seconds: std::env::var("CURSOR_AGENT_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
            max_retries: std::env::var("CURSOR_AGENT_MAX_RETRIES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2),
            working_dir: std::env::var("CURSOR_AGENT_WORKING_DIR").ok(),
            output_format,
            api_key: std::env::var("CURSOR_API_KEY").ok(),
        }
    }
}

#[derive(Debug)]
pub struct CursorAgent {
    config: CursorAgentConfig,
}

impl CursorAgent {
    pub fn with_config(config: CursorAgentConfig) -> Self {
        Self { config }
    }

    pub async fn analyze_code(
        &self,
        request: CodeAnalysisRequest,
        msg_store: Arc<MsgStore>,
        database: Arc<Database>,
    ) -> Result<CodeAnalysisResponse> {
        info!("🚀 Bắt đầu phân tích code cho ticket: {}", request.ticket_id);

        // Check if ticket exists, auto-create if not to prevent FK constraint failure
        let ticket = database.get_ticket(&request.ticket_id).await?;
        if ticket.is_none() {
            info!("🔧 Ticket {} chưa tồn tại, tự động tạo ticket", request.ticket_id);
            
            // Auto-create ticket to prevent FK constraint failure
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
            };
            
            database.create_ticket(&auto_ticket).await?;
            info!("✅ Đã tự động tạo ticket: {}", request.ticket_id);
        }

        // Create analysis session in database
        let session_id = database.create_session(&request.ticket_id).await?;

        // Update ticket status to analyzing
        database
            .update_ticket_analyzing(&request.ticket_id, true)
            .await?;

        let mut logs = Vec::new();
        let normalizer = LogNormalizer::new();

        // Send initial log
        let start_log = "🔄 Khởi động Cursor Agent...";
        let entry = normalizer.normalize(
            start_log.to_string(),
            request.ticket_id.clone(),
        );
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

        // Execute Cursor Agent analysis
        let result = match self
            .execute_cursor_agent(&request, working_directory, &msg_store, &normalizer)
            .await
        {
            Ok(output) => {
                info!("✅ Cursor Agent hoàn thành phân tích");

                // Send completion log with special result type
                let completion_log = "✅ Phân tích hoàn tất!";
                let mut entry = normalizer.normalize(
                    completion_log.to_string(),
                    request.ticket_id.clone(),
                );
                // Override message type to 'result' for completion
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
                error!("❌ Lỗi khi thực thi Cursor Agent: {}", e);

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

    async fn execute_cursor_agent(
        &self,
        request: &CodeAnalysisRequest,
        working_directory: Option<String>,
        msg_store: &Arc<MsgStore>,
        normalizer: &LogNormalizer,
    ) -> Result<String> {
        info!("🎯 Executing analysis for: {}", request.code_context);
        
        // Validate working directory and code_context path
        let analysis_dir = working_directory.or(self.config.working_dir.clone());
        if let Some(ref dir) = analysis_dir {
            info!("📂 Analysis scope: {}", dir);
            // Validate directory exists and is accessible
            if let Err(e) = tokio::fs::metadata(dir).await {
                error!("⚠️ Không thể access directory {}: {}", dir, e);
                return Err(CursorAgentError::DirectoryNotAccessible(dir.clone()).into());
            }
        }

        // Validate executable exists only for absolute paths
        // For executables in PATH, let spawn() handle the error
        if self.config.executable_path.contains('/') || self.config.executable_path.contains('\\') {
            // It's an absolute path, check if exists
            if let Err(_e) = tokio::fs::metadata(&self.config.executable_path).await {
                error!("⚠️ Cursor Agent executable không tồn tại: {}", self.config.executable_path);
                return Err(CursorAgentError::ExecutableNotFound(self.config.executable_path.clone()).into());
            }
        } else {
            // For PATH executables, check if command exists using 'which'
            debug!("Checking if '{}' exists in PATH", self.config.executable_path);
            // Note: On Windows, this might need different handling
            if std::cfg!(unix) {
                if let Ok(output) = tokio::process::Command::new("which")
                    .arg(&self.config.executable_path)
                    .output()
                    .await
                {
                    if !output.status.success() {
                        error!("⚠️ Cursor Agent '{}' không tìm thấy trong PATH", self.config.executable_path);
                        error!("💡 Hãy install Cursor CLI: curl https://cursor.com/install -fsS | bash");
                        error!("💡 Hoặc set CURSOR_AGENT_PATH với absolute path đến executable");
                        return Err(CursorAgentError::ExecutableNotFound(format!("'{}' not found in PATH", self.config.executable_path)).into());
                    }
                }
            }
        }

        // Execute with retry logic
        let mut last_error = None;
        for attempt in 1..=self.config.max_retries {
            info!("🔄 Attempt {}/{} for analysis", attempt, self.config.max_retries);
            
            match self.spawn_cursor_process(request, analysis_dir.clone(), msg_store, normalizer).await {
                Ok(result) => {
                    info!("✅ Analysis completed successfully on attempt {}", attempt);
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


    async fn spawn_cursor_process(
        &self,
        request: &CodeAnalysisRequest,
        working_directory: Option<String>,
        msg_store: &Arc<MsgStore>,
        _normalizer: &LogNormalizer,
    ) -> Result<String> {
        let prompt = self.create_analysis_prompt(request);
        let ticket_id = request.ticket_id.clone();

        info!("🚀 Spawning Cursor Agent process: {}", self.config.executable_path);
        debug!("Prompt: {}", prompt);

        // Build command with proper Cursor CLI arguments according to documentation
        // Reference: https://cursor.com/docs/cli/headless
        let mut cmd = Command::new(&self.config.executable_path);
        
        // Print mode for non-interactive scripting (use either -p OR --print, not both)
        cmd.arg("-p");
        
        // Add output format
        match self.config.output_format {
            OutputFormat::Text => {
                // Default text format, no additional flag needed
            }
            OutputFormat::Json => {
                cmd.arg("--output-format").arg("json");
            }
            OutputFormat::StreamJson => {
                cmd.arg("--output-format").arg("stream-json");
            }
            OutputFormat::StreamPartialOutput => {
                cmd.arg("--output-format").arg("stream-json");
                cmd.arg("--stream-partial-output");
            }
        }
        
        // Set working directory using Rust's Command::current_dir()
        // Cursor CLI will execute in the specified directory context
        if let Some(ref dir) = working_directory {
            cmd.current_dir(dir);
        }
        
        // Add the actual prompt/command as the final argument
        cmd.arg(&prompt);

        // Set API key if available
        if let Some(ref api_key) = self.config.api_key {
            cmd.env("CURSOR_API_KEY", api_key);
        }

        cmd.stdin(std::process::Stdio::piped());  // Key fix: pipe stdin to close it later
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Spawn the process
        let mut child = cmd.spawn()
            .map_err(|e| CursorAgentError::SpawnFailed(e.to_string()))?;

        // Close stdin immediately to signal EOF
        // This forces Cursor Agent to exit after processing instead of waiting for more input
        let _stdin = child.stdin.take();
        drop(_stdin);
        info!("🔒 Closed stdin to signal EOF to Cursor Agent");

        let stdout = child.stdout.take().ok_or_else(|| 
            CursorAgentError::SpawnFailed("Failed to get stdout pipe".to_string()))?;
        let stderr = child.stderr.take().ok_or_else(|| 
            CursorAgentError::SpawnFailed("Failed to get stderr pipe".to_string()))?;

        // Clone for async tasks
        let msg_store_clone = msg_store.clone();
        let ticket_id_clone = ticket_id.clone();

        // Spawn task to capture stdout
        let stdout_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            let mut output_lines = Vec::new();
            let normalizer = LogNormalizer::new();

            while let Ok(Some(line)) = lines.next_line().await {
                info!("📤 STDOUT: {}", line);
                output_lines.push(line.clone());
                
                let entry = normalizer.normalize(line, ticket_id_clone.clone());
                msg_store_clone.push(entry).await;
            }

            info!("📤 Finished reading stdout, total lines: {}", output_lines.len());

            output_lines
        });

        // Spawn task to capture stderr
        let stderr_ticket_id = request.ticket_id.clone();
        let stderr_msg_store = msg_store.clone();

        let stderr_handle = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            let stderr_normalizer = LogNormalizer::new();

            while let Ok(Some(line)) = lines.next_line().await {
                info!("⚠️ STDERR: {}", line);
                let error_line = format!("ERROR: {}", line);
                let entry = stderr_normalizer.normalize(error_line, stderr_ticket_id.clone());
                stderr_msg_store.push(entry).await;
            }

            info!("⚠️ Finished reading stderr");
        });

        // Wait for process to complete with timeout
        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
        info!("⏳ Waiting for Cursor Agent process to complete (timeout: {}s)...", self.config.timeout_seconds);
        
        let process_result = timeout(timeout_duration, child.wait()).await;

        match process_result {
            Ok(Ok(status)) => {
                info!("✅ Cursor Agent process completed with exit code: {}", status.code().unwrap_or(-1));
                
                // Wait for log capture to complete
                let (stdout_result, _) = tokio::join!(stdout_handle, stderr_handle);
                
                let output_lines = stdout_result.map_err(|e| 
                    CursorAgentError::SpawnFailed(format!("Stdout task failed: {}", e)))?;
                
                if !status.success() {
                    return Err(CursorAgentError::ProcessFailed(status.code().unwrap_or(-1)).into());
                }

                if output_lines.is_empty() {
                    warn!("⚠️ Cursor Agent produced no output");
                    return Ok("Analysis completed but no output generated".to_string());
                }

                Ok(output_lines.join("\n"))
            }
            Ok(Err(e)) => {
                error!("❌ Process wait failed: {}", e);
                // Cleanup tasks
                stdout_handle.abort();
                stderr_handle.abort();
                Err(CursorAgentError::SpawnFailed(e.to_string()).into())
            }
            Err(_) => {
                error!("⏰ Process timeout after {} seconds", self.config.timeout_seconds);
                
                // Kill the process
                if let Err(e) = child.kill().await {
                    error!("Failed to kill timeout process: {}", e);
                }
                
                // Cleanup tasks
                stdout_handle.abort();
                stderr_handle.abort();
                
                Err(CursorAgentError::Timeout(self.config.timeout_seconds).into())
            }
        }
    }

    fn create_analysis_prompt(&self, request: &CodeAnalysisRequest) -> String {
        // Create prompt that works with Cursor CLI
        // The prompt should be a natural language instruction
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

// Implement CodeAgent trait for CursorAgent
#[async_trait]
impl CodeAgent for CursorAgent {
    async fn analyze_code(
        &self,
        request: CodeAnalysisRequest,
        msg_store: Arc<MsgStore>,
        database: Arc<Database>,
    ) -> Result<CodeAnalysisResponse> {
        // Delegate to existing implementation
        self.analyze_code(request, msg_store, database).await
    }
}
