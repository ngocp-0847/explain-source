use crate::code_agent::CodeAgent;
use crate::cursor_agent::{CursorAgent, CursorAgentConfig};
use crate::gemini_agent::{GeminiAgent, GeminiAgentConfig};
use std::sync::Arc;
use tracing::info;

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
/// Defaults to Gemini if not specified or if the value is invalid.
pub fn create_agent_from_env() -> Arc<dyn CodeAgent> {
    let agent_type = std::env::var("AGENT_TYPE")
        .ok()
        .and_then(|s| AgentType::from_str(&s))
        .unwrap_or(AgentType::Gemini);

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
