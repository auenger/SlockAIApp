//! Agent runtime module.
//!
//! Provides the abstraction layer for LLM agent runtimes
//! (Claude Code, Codex, etc.).

pub mod a2a;
pub mod claude;
pub mod codex;
#[cfg(feature = "tauri-app")]
pub mod commands;
pub mod registry;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;

// ===========================================================================
// Runtime type enum
// ===========================================================================

/// The type of runtime backend an agent is bound to.
///
/// Each agent is associated with exactly one runtime type, which determines
/// which CLI/API client is used for execution. The default is `ClaudeCode`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeType {
    /// Anthropic Claude Code CLI
    ClaudeCode,
    /// OpenAI Codex CLI
    Codex,
    /// Google Gemini CLI/API
    Gemini,
    /// Extensible custom runtime (name identifies the binary)
    Custom(String),
}

impl RuntimeType {
    /// Returns the kebab-case string used for serialization.
    pub fn as_str(&self) -> &str {
        match self {
            Self::ClaudeCode => "claude_code",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::Custom(name) => name.as_str(),
        }
    }

    /// Returns the runtime ID used for registry lookups.
    pub fn runtime_id(&self) -> &str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::Custom(name) => name.as_str(),
        }
    }

    /// Returns the human-readable display name.
    pub fn display_name(&self) -> &str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::Codex => "OpenAI Codex",
            Self::Gemini => "Google Gemini",
            Self::Custom(name) => name.as_str(),
        }
    }
}

impl Default for RuntimeType {
    fn default() -> Self {
        Self::ClaudeCode
    }
}

impl std::fmt::Display for RuntimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ===========================================================================
// Runtime status & capability types
// ===========================================================================

/// Status of an agent runtime.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRuntimeStatus {
    /// CLI is detected and healthy
    Available,
    /// CLI was detected but failed health check
    Unhealthy,
    /// CLI is not installed on this system
    NotInstalled,
    /// Detection is in progress
    Detecting,
}

impl AgentRuntimeStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Available => "available",
            Self::Unhealthy => "unhealthy",
            Self::NotInstalled => "not-installed",
            Self::Detecting => "detecting",
        }
    }
}

/// Capabilities that a runtime can support.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentCapability {
    Streaming,
    Sessions,
    ToolUse,
    StructuredOutput,
}

// ===========================================================================
// Runtime info & execution types
// ===========================================================================

/// Information about a registered agent runtime.
#[derive(Debug, Clone, Serialize)]
pub struct AgentRuntimeInfo {
    /// Unique runtime identifier (e.g. "claude-code")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Runtime category ("cli" or "http")
    pub runtime_category: String,
    /// The typed runtime type enum
    pub runtime_type: RuntimeType,
    /// Current status
    pub status: String,
    /// Detected version string (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Installation path (if detected)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_path: Option<String>,
    /// Supported capabilities
    pub capabilities: Vec<AgentCapability>,
    /// Install hint command (shown when not-installed)
    pub install_hint: String,
    /// The CLI binary name (e.g. "claude", "codex")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_name: Option<String>,
}

/// Parameters for executing a message on an AgentRuntime.
#[derive(Debug, Clone)]
pub struct ExecuteParams {
    /// The agent ID this execution is for (used for process pool key)
    pub agent_id: String,
    /// The user message to send to the agent
    pub message: String,
    /// Optional session ID for resuming a conversation
    pub session_id: Option<String>,
    /// Optional workspace directory for the agent to operate in
    pub workspace: Option<String>,
    /// Optional system prompt to append
    pub system_prompt: Option<String>,
    /// Maximum execution time in seconds before idle timeout
    pub timeout_secs: u64,
    /// Whether to use a persistent process (Thread) or one-shot (Channel).
    /// Thread mode: reuse process via stdin JSON, context kept in-process.
    /// Channel mode: fresh process each request, context reconstructed by caller.
    pub persistent: bool,
    /// Optional MCP (Model Context Protocol) configuration to inject.
    /// When present, written to a temp file and passed via --mcp-config + --strict-mcp-config.
    /// Format: JSON object with `mcpServers` key, e.g.:
    /// ```json
    /// { "mcpServers": { "filesystem": { "command": "npx", "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path"] } } }
    /// ```
    pub mcp_config: Option<serde_json::Value>,
}

/// Token usage statistics for a single model, accumulated across all assistant
/// responses in a conversation. Used for cost tracking and display.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TokenUsage {
    /// Number of input tokens sent to the model.
    pub input_tokens: u64,
    /// Number of output tokens generated by the model.
    pub output_tokens: u64,
    /// Number of tokens read from the prompt cache (Anthropic-specific).
    pub cache_read_tokens: u64,
    /// Number of tokens written to the prompt cache (Anthropic-specific).
    pub cache_write_tokens: u64,
}

impl TokenUsage {
    /// Create a zero-valued TokenUsage.
    pub fn zero() -> Self {
        Self::default()
    }

    /// Merge another TokenUsage into this one, adding all fields.
    pub fn merge(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.cache_read_tokens += other.cache_read_tokens;
        self.cache_write_tokens += other.cache_write_tokens;
    }

    /// Extract TokenUsage from a Claude SDK `message.usage` JSON object.
    ///
    /// Expected fields: `input_tokens`, `output_tokens`,
    /// `cache_read_input_tokens`, `cache_creation_input_tokens`.
    /// Missing fields default to 0.
    pub fn from_claude_usage(usage: &serde_json::Value) -> Self {
        Self {
            input_tokens: usage.get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            output_tokens: usage.get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            cache_read_tokens: usage.get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            cache_write_tokens: usage.get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        }
    }
}

/// Merge two token usage maps. For each model key present in `source`,
/// add its values to the entry in `target` (or insert if absent).
pub fn merge_token_usage_maps(
    target: &mut HashMap<String, TokenUsage>,
    source: &HashMap<String, TokenUsage>,
) {
    for (model, usage) in source {
        target
            .entry(model.clone())
            .or_default()
            .merge(usage);
    }
}

/// A streaming event produced by an AgentRuntime during execution.
#[derive(Debug, Clone, Serialize)]
pub struct StreamEvent {
    /// Text content of the event
    pub text: String,
    /// Whether this is the final event for this execution
    pub is_done: bool,
    /// Error message if something went wrong
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The type of this event: "assistant", "result", "system", "raw", "stderr", "timeout"
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub msg_type: Option<String>,
    /// Session ID from the CLI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Structured content blocks (tool_use, tool_result) from verbose JSON output.
    /// When present, the frontend can render tool calls as structured cards.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_blocks: Option<serde_json::Value>,
    /// Accumulated token usage by model name (e.g. "claude-sonnet-4-6").
    /// Populated from assistant message `usage` fields during streaming.
    /// Only present on the final `is_done` event (result type).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<HashMap<String, TokenUsage>>,
}

// ===========================================================================
// AgentRuntime trait
// ===========================================================================

/// Trait that all agent runtimes must implement.
/// Each runtime wraps a specific AI coding CLI tool.
pub trait AgentRuntime: Send + Sync {
    /// Unique runtime identifier (e.g. "claude-code")
    fn id(&self) -> &str;
    /// Human-readable name
    fn name(&self) -> &str;
    /// Runtime category (e.g. "cli")
    fn runtime_category(&self) -> &str {
        "cli"
    }
    /// The typed RuntimeType enum variant for this runtime.
    fn typed_runtime_type(&self) -> RuntimeType;
    /// List of capabilities this runtime supports
    fn capabilities(&self) -> Vec<AgentCapability>;
    /// Install hint command for this runtime
    fn install_hint(&self) -> String;
    /// The CLI binary name (e.g. "claude", "codex")
    fn binary_name(&self) -> &str;
    /// Detect whether the CLI is available and return (path, version)
    fn detect(&self) -> Result<Option<(String, String)>, String>;
    /// Perform a health check
    fn health_check(&self) -> AgentRuntimeStatus;
    /// Get the current runtime info (id, name, status, version, etc.)
    fn info(&self) -> AgentRuntimeInfo;
    /// Check if the runtime is ready for execution
    fn is_ready(&self) -> bool;
    /// Execute a message and return a receiver for streaming events.
    fn execute(&self, params: ExecuteParams) -> Result<Receiver<StreamEvent>, String>;
}

// ===========================================================================
// CLI detector utility
// ===========================================================================

/// Detector utility: checks if a CLI command exists on PATH and gets its version.
pub struct RuntimeDetector;

impl RuntimeDetector {
    /// Check if a command exists on the system PATH.
    /// Returns the full path if found.
    pub fn find_command(cmd: &str) -> Option<String> {
        #[cfg(unix)]
        {
            std::process::Command::new("which")
                .arg(cmd)
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| {
                    let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if path.is_empty() { None } else { Some(path) }
                })
        }
        #[cfg(windows)]
        {
            std::process::Command::new("where")
                .arg(cmd)
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| {
                    let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if path.is_empty() { None } else { Some(path) }
                })
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = cmd;
            None
        }
    }

    /// Get the version string for a CLI command via `<cmd> --version`.
    pub fn get_version(cmd: &str) -> Option<String> {
        std::process::Command::new(cmd)
            .arg("--version")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }
}
