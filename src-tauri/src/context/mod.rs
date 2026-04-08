//! Context orchestration module.
//!
//! Responsible for assembling, compressing, and managing
//! the conversation history that gets sent to Agent runtimes.
//!
//! ## Context Assembly
//!
//! When building context for an Agent, the engine loads:
//! 1. **Global SOUL.md** -- default personality (from workspace root)
//! 2. **Agent SOUL.md** -- overrides global if present
//! 3. **IDENTITY.md** -- Agent metadata (name, emoji, vibe)
//! 4. **USER.md** -- user preferences
//! 5. **AGENTS.md** -- behavior instructions
//! 6. **TOOLS.md** -- tool usage guide
//! 7. **memory/MEMORY.md** -- long-term memory
//! 8. **memory/HISTORY.md** -- conversation summaries

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Assembled context for an Agent, ready to be sent to the runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// Agent ID this context was built for.
    pub agent_id: String,
    /// The system prompt (assembled from SOUL.md + IDENTITY.md).
    pub system_prompt: String,
    /// User profile context (from USER.md).
    pub user_context: Option<String>,
    /// Agent behavior instructions (from AGENTS.md).
    pub agent_instructions: Option<String>,
    /// Tool usage guide (from TOOLS.md).
    pub tool_instructions: Option<String>,
    /// Long-term memory (from memory/MEMORY.md).
    pub memory: Option<String>,
    /// History summary (from memory/HISTORY.md).
    pub history_summary: Option<String>,
}

/// Context builder that assembles Agent context from workspace files.
pub struct ContextBuilder {
    /// Workspace root path (contains global templates).
    workspace_root: PathBuf,
    /// Agents directory path.
    agents_dir: PathBuf,
}

impl ContextBuilder {
    /// Create a new ContextBuilder for the given workspace root.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root = workspace_root.into();
        Self {
            agents_dir: root.join("agents"),
            workspace_root: root,
        }
    }

    /// Build the full context for a specific Agent.
    ///
    /// Loads Agent-level SOUL.md (if present, overrides global),
    /// IDENTITY.md, and all supporting context files.
    pub fn build(&self, agent_id: &str) -> Result<AgentContext, ContextError> {
        let agent_dir = self.agents_dir.join(agent_id);

        if !agent_dir.is_dir() {
            return Err(ContextError::AgentNotFound {
                agent_id: agent_id.to_string(),
            });
        }

        // Load SOUL.md: agent-level overrides global
        let soul_content = self.load_soul(&agent_dir);

        // Load IDENTITY.md
        let identity_content = read_file_optional(&agent_dir.join("IDENTITY.md"));

        // Build system prompt from SOUL + IDENTITY
        let system_prompt = self.build_system_prompt(&soul_content, identity_content.as_ref());

        // Load supporting context files
        let user_context = read_file_optional(&self.workspace_root.join("USER.md"));
        let agent_instructions = read_file_optional(&self.workspace_root.join("AGENTS.md"));
        let tool_instructions = read_file_optional(&self.workspace_root.join("TOOLS.md"));
        let memory = read_file_optional(&self.workspace_root.join("memory/MEMORY.md"));
        let history_summary = read_file_optional(&self.workspace_root.join("memory/HISTORY.md"));

        Ok(AgentContext {
            agent_id: agent_id.to_string(),
            system_prompt,
            user_context,
            agent_instructions,
            tool_instructions,
            memory,
            history_summary,
        })
    }

    /// Load the effective SOUL.md for an Agent.
    ///
    /// Priority: Agent-level SOUL.md > Global SOUL.md
    fn load_soul(&self, agent_dir: &Path) -> String {
        // Try Agent-level first
        let agent_soul = agent_dir.join("SOUL.md");
        if agent_soul.exists() {
            if let Ok(content) = fs::read_to_string(&agent_soul) {
                return content;
            }
        }

        // Fallback to global
        let global_soul = self.workspace_root.join("SOUL.md");
        if let Ok(content) = fs::read_to_string(&global_soul) {
            return content;
        }

        // Ultimate fallback
        "# Soul\n\nYou are a helpful AI assistant.\n".to_string()
    }

    /// Build the system prompt from SOUL.md and IDENTITY.md content.
    fn build_system_prompt(&self, soul: &str, identity: Option<&String>) -> String {
        let mut prompt = String::new();

        // Add identity prefix if available
        if let Some(id_content) = identity {
            // Extract key identity fields for the prompt
            prompt.push_str("# Your Identity\n\n");
            prompt.push_str(id_content);
            prompt.push_str("\n\n---\n\n");
        }

        // Add soul content
        prompt.push_str(soul);

        prompt
    }

    /// Get just the SOUL.md content for an Agent (for quick access).
    pub fn get_soul(&self, agent_id: &str) -> Option<String> {
        let agent_dir = self.agents_dir.join(agent_id);
        if agent_dir.is_dir() {
            Some(self.load_soul(&agent_dir))
        } else {
            None
        }
    }

    /// Get the global USER.md content.
    pub fn get_user_profile(&self) -> Option<String> {
        read_file_optional(&self.workspace_root.join("USER.md"))
    }

    /// Build a context prefix string suitable for prepending to conversations.
    ///
    /// Returns a formatted string containing the essential context elements.
    pub fn build_context_prefix(&self, agent_id: &str) -> Result<String, ContextError> {
        let ctx = self.build(agent_id)?;

        let mut prefix = String::new();

        prefix.push_str(&ctx.system_prompt);
        prefix.push_str("\n\n");

        if let Some(ref user) = ctx.user_context {
            prefix.push_str("# User Profile\n\n");
            prefix.push_str(user);
            prefix.push_str("\n\n");
        }

        if let Some(ref instructions) = ctx.agent_instructions {
            prefix.push_str("# Agent Instructions\n\n");
            prefix.push_str(instructions);
            prefix.push_str("\n\n");
        }

        if let Some(ref tools) = ctx.tool_instructions {
            prefix.push_str("# Tool Usage\n\n");
            prefix.push_str(tools);
            prefix.push_str("\n\n");
        }

        if let Some(ref memory) = ctx.memory {
            prefix.push_str("# Memory\n\n");
            prefix.push_str(memory);
            prefix.push_str("\n\n");
        }

        Ok(prefix)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read a file if it exists, returning None if not found.
fn read_file_optional(path: &Path) -> Option<String> {
    if path.exists() {
        fs::read_to_string(path).ok()
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("agent not found: {agent_id}")]
    AgentNotFound { agent_id: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::manager::AgentManager;

    #[test]
    fn test_build_context_with_agent_soul_override() {
        let dir = tempfile::tempdir().unwrap();

        // Initialize workspace
        let mut manager = AgentManager::new(dir.path());
        manager.initialize_workspace().unwrap();
        manager.load().unwrap();
        manager.create_agent("Claude", "AI", "sharp", "sparkles", None).unwrap();

        // Override agent SOUL.md
        let agent_soul = dir.path().join("agents/claude/SOUL.md");
        fs::write(&agent_soul, "# Claude's Soul\n\nI am sharp and witty.\n").unwrap();

        // Build context
        let builder = ContextBuilder::new(dir.path());
        let ctx = builder.build("claude").unwrap();

        // Agent-level SOUL.md should be used (not global)
        assert!(ctx.system_prompt.contains("Claude's Soul"));
        assert!(ctx.system_prompt.contains("sharp and witty"));
        assert!(!ctx.system_prompt.contains("Core Truths"));
    }

    #[test]
    fn test_build_context_uses_agent_soul_or_global() {
        let dir = tempfile::tempdir().unwrap();

        let mut manager = AgentManager::new(dir.path());
        manager.initialize_workspace().unwrap();
        manager.load().unwrap();

        // Default agent gets its own SOUL.md during creation
        let builder = ContextBuilder::new(dir.path());
        let ctx = builder.build("default").unwrap();

        // Agent-level SOUL.md is used (personalized, contains "SlockAI")
        assert!(ctx.system_prompt.contains("SlockAI") || ctx.system_prompt.contains("Soul"));
    }

    #[test]
    fn test_context_prefix_includes_user_profile() {
        let dir = tempfile::tempdir().unwrap();

        let mut manager = AgentManager::new(dir.path());
        manager.initialize_workspace().unwrap();
        manager.load().unwrap();

        // Write custom USER.md
        fs::write(dir.path().join("USER.md"), "# User\nName: Test User\n").unwrap();

        let builder = ContextBuilder::new(dir.path());
        let prefix = builder.build_context_prefix("default").unwrap();

        assert!(prefix.contains("Test User"));
    }

    #[test]
    fn test_agent_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let builder = ContextBuilder::new(dir.path());
        let result = builder.build("nonexistent");
        assert!(result.is_err());
    }
}
