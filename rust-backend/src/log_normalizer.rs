use crate::message_store::{LogMessageType, StructuredLogEntry};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

pub struct LogNormalizer {
    // Pre-compiled regex patterns for performance
    file_path_pattern: Regex,
    error_pattern: Regex,
    tool_pattern: Regex,
    line_number_pattern: Regex,
}

impl LogNormalizer {
    pub fn new() -> Self {
        Self {
            // Match file paths like "path/to/file.js" or "/absolute/path.ts"
            file_path_pattern: Regex::new(r#"(?:Reading|Analyzing|Processing|File:)\s+([^\s]+\.[a-zA-Z]{1,4})"#).unwrap(),

            // Match error codes and severity levels
            error_pattern: Regex::new(r#"(ERROR|WARN|WARNING|CRITICAL|FATAL)(?::\s*)?(.*)?"#).unwrap(),

            // Match tool usage patterns
            tool_pattern: Regex::new(r#"(?:Using tool|Tool:|Executing):\s*(\w+)"#).unwrap(),

            // Match line numbers
            line_number_pattern: Regex::new(r#"line[s]?\s*(\d+)"#).unwrap(),
        }
    }

    pub fn normalize(&self, raw_log: String, ticket_id: String) -> StructuredLogEntry {
        // Check if this is a JSON log (from Gemini CLI or Cursor Agent)
        let (message_type, content, metadata) = if let Ok(json_value) = serde_json::from_str::<Value>(&raw_log) {
            // This is a JSON log, parse it
            self.normalize_json_log(json_value, &raw_log)
        } else {
            // Plain text log, use existing logic
            let message_type = self.classify(&raw_log);
            let content = self.clean_content(&raw_log, &message_type);
            let metadata = self.extract_metadata(&raw_log, &message_type);
            (message_type, content, metadata)
        };

        StructuredLogEntry {
            id: Uuid::new_v4().to_string(),
            ticket_id,
            message_type,
            content,
            raw_log: Some(raw_log),
            metadata,
            timestamp: chrono::Utc::now(),
        }
    }

    fn normalize_json_log(&self, json_value: Value, raw_log: &str) -> (LogMessageType, String, HashMap<String, String>) {
        let mut metadata = HashMap::new();
        
        // Extract type and role from JSON
        let msg_type = json_value.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let role = json_value.get("role").and_then(|v| v.as_str()).unwrap_or("");
        
        // Classify based on JSON structure
        let message_type = match (msg_type, role) {
            ("message", "assistant") => LogMessageType::Assistant,
            ("message", "user") => LogMessageType::System,
            ("tool_use", _) => LogMessageType::ToolUse,
            ("tool_result", _) => LogMessageType::System,
            ("init", _) => LogMessageType::System,
            ("error", _) | (_, _) if json_value.get("error").is_some() 
                || json_value.get("status").and_then(|v| v.as_str()) == Some("error") => LogMessageType::Error,
            _ => LogMessageType::System,
        };

        // Extract metadata from JSON
        if let Some(tool_name) = json_value.get("tool_name").and_then(|v| v.as_str()) {
            metadata.insert("tool_name".to_string(), tool_name.to_string());
        }
        if let Some(tool_id) = json_value.get("tool_id").and_then(|v| v.as_str()) {
            metadata.insert("tool_id".to_string(), tool_id.to_string());
        }
        if let Some(timestamp) = json_value.get("timestamp").and_then(|v| v.as_str()) {
            metadata.insert("timestamp".to_string(), timestamp.to_string());
        }
        if let Some(session_id) = json_value.get("session_id").and_then(|v| v.as_str()) {
            metadata.insert("session_id".to_string(), session_id.to_string());
        }
        if let Some(model) = json_value.get("model").and_then(|v| v.as_str()) {
            metadata.insert("model".to_string(), model.to_string());
        }

        // Extract content - keep JSON structure intact for LogViewer to parse
        // LogViewer expects: {type, role, content} or {content: [...]}
        let content = raw_log.to_string(); // Keep original JSON string

        (message_type, content, metadata)
    }

    fn classify(&self, log: &str) -> LogMessageType {
        let log_lower = log.to_lowercase();

        // Check for errors first (highest priority)
        if self.error_pattern.is_match(log)
            || log_lower.contains("error")
            || log_lower.contains("failed")
            || log_lower.contains("exception") {
            return LogMessageType::Error;
        }

        // Check for tool usage
        if self.tool_pattern.is_match(log)
            || log_lower.contains("reading file")
            || log_lower.contains("analyzing")
            || log_lower.contains("processing")
            || log_lower.contains("searching")
            || log_lower.contains("executing") {
            return LogMessageType::ToolUse;
        }

        // Check for assistant responses
        if log_lower.starts_with("analysis:")
            || log_lower.starts_with("found:")
            || log_lower.starts_with("result:")
            || log_lower.starts_with("summary:")
            || log_lower.contains("explanation:")
            || log_lower.contains("business flow")
            || log_lower.contains("test case") {
            return LogMessageType::Assistant;
        }

        // Default to system log
        LogMessageType::System
    }

    fn clean_content(&self, raw_log: &str, message_type: &LogMessageType) -> String {
        let mut content = raw_log.trim().to_string();

        // Remove ANSI color codes
        content = self.remove_ansi_codes(&content);

        // Remove excessive whitespace
        content = content.split_whitespace().collect::<Vec<&str>>().join(" ");

        // Remove common log prefixes based on message type
        match message_type {
            LogMessageType::Error => {
                content = content
                    .replace("ERROR:", "")
                    .replace("WARN:", "")
                    .replace("WARNING:", "")
                    .trim()
                    .to_string();
            }
            LogMessageType::ToolUse => {
                content = content
                    .replace("Using tool:", "")
                    .replace("Tool:", "")
                    .replace("Executing:", "")
                    .trim()
                    .to_string();
            }
            _ => {}
        }

        content
    }

    fn remove_ansi_codes(&self, text: &str) -> String {
        // Remove ANSI escape sequences (color codes, cursor movements, etc.)
        let ansi_pattern = Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap();
        ansi_pattern.replace_all(text, "").to_string()
    }

    fn extract_metadata(&self, log: &str, message_type: &LogMessageType) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        match message_type {
            LogMessageType::ToolUse => {
                // Extract file path
                if let Some(caps) = self.file_path_pattern.captures(log) {
                    if let Some(file_path) = caps.get(1) {
                        metadata.insert("file_path".to_string(), file_path.as_str().to_string());

                        // Extract file extension
                        if let Some(ext) = file_path.as_str().split('.').last() {
                            metadata.insert("file_extension".to_string(), ext.to_string());
                        }
                    }
                }

                // Extract line numbers
                if let Some(caps) = self.line_number_pattern.captures(log) {
                    if let Some(line_num) = caps.get(1) {
                        metadata.insert("line_number".to_string(), line_num.as_str().to_string());
                    }
                }

                // Extract tool name
                if let Some(caps) = self.tool_pattern.captures(log) {
                    if let Some(tool_name) = caps.get(1) {
                        metadata.insert("tool_name".to_string(), tool_name.as_str().to_string());
                    }
                }
            }

            LogMessageType::Error => {
                // Extract error severity and message
                if let Some(caps) = self.error_pattern.captures(log) {
                    if let Some(severity) = caps.get(1) {
                        metadata.insert("severity".to_string(), severity.as_str().to_lowercase());
                    }
                    if let Some(error_msg) = caps.get(2) {
                        let msg = error_msg.as_str().trim();
                        if !msg.is_empty() {
                            metadata.insert("error_message".to_string(), msg.to_string());
                        }
                    }
                }

                // Try to extract error code (e.g., "E001", "ERR_123")
                let error_code_pattern = Regex::new(r"(?:E|ERR)[-_]?\d{3,4}").unwrap();
                if let Some(caps) = error_code_pattern.captures(log) {
                    if let Some(code) = caps.get(0) {
                        metadata.insert("error_code".to_string(), code.as_str().to_string());
                    }
                }
            }

            LogMessageType::Assistant => {
                // Extract analysis type
                let analysis_types = vec![
                    "business flow",
                    "test case",
                    "code review",
                    "security",
                    "performance",
                ];

                for analysis_type in analysis_types {
                    if log.to_lowercase().contains(analysis_type) {
                        metadata.insert("analysis_type".to_string(), analysis_type.to_string());
                        break;
                    }
                }
            }

            LogMessageType::System => {
                // Extract progress indicators
                if log.contains("%") {
                    let percent_pattern = Regex::new(r"(\d+)%").unwrap();
                    if let Some(caps) = percent_pattern.captures(log) {
                        if let Some(progress) = caps.get(1) {
                            metadata.insert("progress".to_string(), progress.as_str().to_string());
                        }
                    }
                }

                // Extract duration/time information
                let duration_pattern = Regex::new(r"(\d+(?:\.\d+)?)\s*(ms|seconds?|minutes?|s|m)").unwrap();
                if let Some(caps) = duration_pattern.captures(log) {
                    if let Some(duration) = caps.get(0) {
                        metadata.insert("duration".to_string(), duration.as_str().to_string());
                    }
                }
            }

            LogMessageType::Result => {
                // Extract completion status and duration
                let completion_pattern = Regex::new(r"(?:completed|finished|done|success)").unwrap();
                if completion_pattern.is_match(log) {
                    metadata.insert("status".to_string(), "completed".to_string());
                }

                // Extract duration/time information for completion
                let duration_pattern = Regex::new(r"(\d+(?:\.\d+)?)\s*(ms|seconds?|minutes?|s|m)").unwrap();
                if let Some(caps) = duration_pattern.captures(log) {
                    if let Some(duration) = caps.get(0) {
                        metadata.insert("duration".to_string(), duration.as_str().to_string());
                    }
                }
            }
        }

        // Common metadata: extract timestamps if present in log
        let timestamp_pattern = Regex::new(r"\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}").unwrap();
        if let Some(caps) = timestamp_pattern.captures(log) {
            if let Some(ts) = caps.get(0) {
                metadata.insert("log_timestamp".to_string(), ts.as_str().to_string());
            }
        }

        metadata
    }
}

impl Default for LogNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_error_log() {
        let normalizer = LogNormalizer::new();

        let log = "ERROR: Failed to read file: permission denied";
        let entry = normalizer.normalize(log.to_string(), "test-ticket".to_string());

        assert!(matches!(entry.message_type, LogMessageType::Error));
        assert!(entry.metadata.contains_key("severity"));
    }

    #[test]
    fn test_classify_tool_log() {
        let normalizer = LogNormalizer::new();

        let log = "Reading file: src/auth/login.js";
        let entry = normalizer.normalize(log.to_string(), "test-ticket".to_string());

        assert!(matches!(entry.message_type, LogMessageType::ToolUse));
        assert_eq!(entry.metadata.get("file_path"), Some(&"src/auth/login.js".to_string()));
    }

    #[test]
    fn test_classify_assistant_log() {
        let normalizer = LogNormalizer::new();

        let log = "Analysis: Found 3 business flow patterns in the authentication system";
        let entry = normalizer.normalize(log.to_string(), "test-ticket".to_string());

        assert!(matches!(entry.message_type, LogMessageType::Assistant));
    }

    #[test]
    fn test_extract_file_metadata() {
        let normalizer = LogNormalizer::new();

        let log = "Analyzing file: api/payment.js on line 45";
        let entry = normalizer.normalize(log.to_string(), "test-ticket".to_string());

        assert_eq!(entry.metadata.get("file_path"), Some(&"api/payment.js".to_string()));
        assert_eq!(entry.metadata.get("file_extension"), Some(&"js".to_string()));
        assert_eq!(entry.metadata.get("line_number"), Some(&"45".to_string()));
    }

    #[test]
    fn test_clean_ansi_codes() {
        let normalizer = LogNormalizer::new();

        let log = "\x1B[32mSUCCESS\x1B[0m: Operation completed";
        let entry = normalizer.normalize(log.to_string(), "test-ticket".to_string());

        assert!(!entry.content.contains("\x1B"));
        assert!(entry.content.contains("SUCCESS"));
    }
}
