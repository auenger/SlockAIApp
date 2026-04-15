//! Codex CLI adapter for A2A protocol.
//!
//! Wraps `CodexRuntime::execute()` into the A2A Task lifecycle.
//! Similar to ClaudeCodeAdapter but tailored for Codex CLI specifics.

use super::cli_adapter::{spawn_status_tracker, AdapterConfig, AdapterState, CliA2AAdapter};
use crate::runtime::codex::CodexRuntime;
use crate::runtime::{AgentRuntime, ExecuteParams};
use crate::runtime::a2a::types::{A2AError, TaskStatus};

use std::sync::{Arc, Mutex};

// ===========================================================================
// CodexAdapter
// ===========================================================================

/// A2A adapter that wraps the Codex CLI runtime.
///
/// Manages A2A task lifecycle for Codex CLI executions. Codex has simpler
/// process management compared to Claude Code (no persistent process pool).
pub struct CodexAdapter {
    /// The underlying Codex runtime.
    runtime: CodexRuntime,
    /// Shared state for tracking task statuses.
    state: Arc<Mutex<AdapterState>>,
}

impl CodexAdapter {
    /// Create a new Codex adapter.
    pub fn new() -> Self {
        Self {
            runtime: CodexRuntime::new(),
            state: AdapterState::shared(),
        }
    }

    /// Create an adapter with a pre-existing runtime instance.
    pub fn with_runtime(runtime: CodexRuntime) -> Self {
        Self {
            runtime,
            state: AdapterState::shared(),
        }
    }
}

impl Default for CodexAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CliA2AAdapter for CodexAdapter {
    fn execute_task(
        &self,
        task_id: &str,
        message: &str,
        config: &AdapterConfig,
    ) -> Result<std::sync::mpsc::Receiver<crate::runtime::StreamEvent>, A2AError> {
        // Mark task as WORKING
        {
            let mut state = self.state.lock().map_err(|e| {
                A2AError::internal_error(format!("State lock poisoned: {}", e))
            })?;
            state.set_task_status(task_id, TaskStatus::Working);
        }

        // Build ExecuteParams from adapter config
        // Note: Codex doesn't support persistent mode; always one-shot
        let params = ExecuteParams {
            agent_id: task_id.to_string(),
            message: message.to_string(),
            session_id: config.session_id.clone(),
            workspace: config.workspace.clone(),
            system_prompt: config.system_prompt.clone(),
            timeout_secs: config.timeout_secs,
            persistent: false, // Codex always uses one-shot mode
        };

        // Execute via the underlying Codex runtime
        let rx = self.runtime.execute(params).map_err(|e| {
            // Mark task as FAILED on execution error
            if let Ok(mut state) = self.state.lock() {
                state.set_task_status(task_id, TaskStatus::Failed);
            }
            A2AError::internal_error(format!("Codex execution failed: {}", e))
        })?;

        // Spawn a status tracker that monitors events and updates shared state
        let tracked_rx = spawn_status_tracker(
            task_id.to_string(),
            rx,
            self.state.clone(),
        );

        Ok(tracked_rx)
    }

    fn cancel_task(&self, task_id: &str) -> Result<(), A2AError> {
        // Codex doesn't have persistent processes to kill.
        // Mark the task as canceled in state.
        {
            let mut state = self.state.lock().map_err(|e| {
                A2AError::internal_error(format!("State lock poisoned: {}", e))
            })?;
            state.set_task_status(task_id, TaskStatus::Canceled);
        }

        log::info!("[CodexAdapter] Task {} canceled", task_id);
        Ok(())
    }

    fn get_task_status(&self, task_id: &str) -> Result<TaskStatus, A2AError> {
        let state = self.state.lock().map_err(|e| {
            A2AError::internal_error(format!("State lock poisoned: {}", e))
        })?;
        state
            .get_task_status(task_id)
            .ok_or_else(|| A2AError::task_not_found(task_id))
    }

    fn runtime_type(&self) -> &str {
        "codex"
    }

    fn runtime_name(&self) -> &str {
        "OpenAI Codex"
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "streaming".to_string(),
            "sessions".to_string(),
            "tool_use".to_string(),
        ]
    }

    fn is_available(&self) -> bool {
        self.runtime.is_ready()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codex_adapter_new() {
        let adapter = CodexAdapter::new();
        assert_eq!(adapter.runtime_type(), "codex");
        assert_eq!(adapter.runtime_name(), "OpenAI Codex");
    }

    #[test]
    fn test_codex_adapter_capabilities() {
        let adapter = CodexAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.contains(&"streaming".to_string()));
        assert!(caps.contains(&"tool_use".to_string()));
    }

    #[test]
    fn test_codex_adapter_get_task_status_not_found() {
        let adapter = CodexAdapter::new();
        let result = adapter.get_task_status("nonexistent");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, -32001); // task_not_found
    }

    #[test]
    fn test_codex_adapter_cancel_task() {
        let adapter = CodexAdapter::new();
        let result = adapter.cancel_task("t-cancel");
        assert!(result.is_ok());
        // Verify status was updated
        let status = adapter.get_task_status("t-cancel").unwrap();
        assert_eq!(status, TaskStatus::Canceled);
    }

    #[test]
    fn test_codex_adapter_default() {
        let adapter = CodexAdapter::default();
        assert_eq!(adapter.runtime_type(), "codex");
    }
}
