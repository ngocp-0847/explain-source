use crate::code_agent::CodeAgent;
use crate::cursor_agent::{CursorAgent, CursorAgentConfig};
use crate::gemini_agent::{GeminiAgent, GeminiAgentConfig};
use std::sync::Arc;
use tracing::{info, warn, debug};

/// Type of code analysis agent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    Gemini,
    Cursor,
}

impl AgentType {
    /// Parse agent type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "gemini" => Some(Self::Gemini),
            "cursor" => Some(Self::Cursor),
            _ => None,
        }
    }

    /// Get agent type name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Gemini => "Gemini CLI",
            Self::Cursor => "Cursor Agent",
        }
    }
}

/// Create a code agent based on the specified type
pub fn create_agent(agent_type: AgentType) -> Arc<dyn CodeAgent> {
    match agent_type {
        AgentType::Gemini => {
            let config = GeminiAgentConfig::from_env();
            info!("🔧 Creating Gemini CLI agent");
            info!("  - Executable: {}", config.executable_path);
            info!("  - Timeout: {}s", config.timeout_seconds);
            info!("  - Retries: {}", config.max_retries);
            info!("  - Output format: {:?}", config.output_format);
            if config.api_key.is_some() {
                info!("  - API key: [SET]");
            }
            Arc::new(GeminiAgent::with_config(config))
        }
        AgentType::Cursor => {
            let config = CursorAgentConfig::from_env();
            info!("🔧 Creating Cursor Agent");
            info!("  - Executable: {}", config.executable_path);
            info!("  - Timeout: {}s", config.timeout_seconds);
            info!("  - Retries: {}", config.max_retries);
            info!("  - Output format: {:?}", config.output_format);
            if config.api_key.is_some() {
                info!("  - API key: [SET]");
            }
            Arc::new(CursorAgent::with_config(config))
        }
    }
}

/// Create a code agent from environment variables
///
/// Reads the `AGENT_TYPE` environment variable to determine which agent to create.
/// **Default: Gemini** - If `AGENT_TYPE` is not set, empty, or has an invalid value,
/// the system will automatically use Gemini Agent as the default.
pub fn create_agent_from_env() -> Arc<dyn CodeAgent> {
    // Read AGENT_TYPE from environment
    let agent_type_env = std::env::var("AGENT_TYPE").ok();
    
    // Debug: log the raw value from environment
    match &agent_type_env {
        Some(val) => {
            debug!("📋 AGENT_TYPE environment variable: '{}'", val);
        }
        None => {
            debug!("📋 AGENT_TYPE environment variable: not set");
        }
    }
    
    // Parse and determine agent type
    let agent_type = agent_type_env
        .as_ref()
        .and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                warn!("⚠️ AGENT_TYPE is set but empty, defaulting to Gemini");
                None
            } else {
                AgentType::from_str(trimmed)
            }
        })
        .or_else(|| {
            // Log when falling back to default
            match &agent_type_env {
                Some(val) => {
                    warn!("⚠️ Invalid AGENT_TYPE value '{}', defaulting to Gemini", val);
                }
                None => {
                    info!("ℹ️ AGENT_TYPE not specified, using default: Gemini");
                }
            }
            Some(AgentType::Gemini)
        })
        .unwrap_or(AgentType::Gemini); // Final fallback (should never reach here)

    info!("🤖 Selected code analysis agent: {}", agent_type.name());

    create_agent(agent_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_from_str() {
        assert_eq!(AgentType::from_str("gemini"), Some(AgentType::Gemini));
        assert_eq!(AgentType::from_str("Gemini"), Some(AgentType::Gemini));
        assert_eq!(AgentType::from_str("GEMINI"), Some(AgentType::Gemini));
        assert_eq!(AgentType::from_str("cursor"), Some(AgentType::Cursor));
        assert_eq!(AgentType::from_str("Cursor"), Some(AgentType::Cursor));
        assert_eq!(AgentType::from_str("CURSOR"), Some(AgentType::Cursor));
        assert_eq!(AgentType::from_str("invalid"), None);
    }

    #[test]
    fn test_agent_type_name() {
        assert_eq!(AgentType::Gemini.name(), "Gemini CLI");
        assert_eq!(AgentType::Cursor.name(), "Cursor Agent");
    }
}
