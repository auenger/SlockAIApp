//! Claude Code CLI adapter for A2A protocol.
//!
//! Wraps `ClaudeCodeRuntime::execute()` into the A2A Task lifecycle.
//! Translates A2A SendMessage requests into `ExecuteParams` and
//! streams CLI output back as A2A-compatible events.

use super::cli_adapter::{spawn_status_tracker, AdapterConfig, AdapterState, CliA2AAdapter};
use crate::runtime::claude::ClaudeCodeRuntime;
use crate::runtime::{AgentRuntime, ExecuteParams};
use crate::runtime::a2a::types::{A2AError, TaskStatus};

use std::sync::{Arc, Mutex};

// ===========================================================================
// ClaudeCodeAdapter
// ===========================================================================

/// A2A adapter that wraps the Claude Code CLI runtime.
///
/// Each adapter instance manages a pool of tasks, mapping A2A Task IDs
/// to underlying Claude Code CLI process executions.
pub struct ClaudeCodeAdapter {
    /// The underlying Claude Code runtime.
    runtime: ClaudeCodeRuntime,
    /// Shared state for tracking task statuses.
    state: Arc<Mutex<AdapterState>>,
}

impl ClaudeCodeAdapter {
    /// Create a new Claude Code adapter.
    pub fn new() -> Self {
        Self {
            runtime: ClaudeCodeRuntime::new(),
            state: AdapterState::shared(),
        }
    }

    /// Create an adapter with a pre-existing runtime instance.
    pub fn with_runtime(runtime: ClaudeCodeRuntime) -> Self {
        Self {
            runtime,
            state: AdapterState::shared(),
        }
    }
}

impl Default for ClaudeCodeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CliA2AAdapter for ClaudeCodeAdapter {
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
        let params = ExecuteParams {
            agent_id: task_id.to_string(),
            message: message.to_string(),
            session_id: config.session_id.clone(),
            workspace: config.workspace.clone(),
            system_prompt: config.system_prompt.clone(),
            timeout_secs: config.timeout_secs,
            persistent: config.persistent,
            mcp_config: None,
        };

        // Execute via the underlying Claude Code runtime
        let rx = self.runtime.execute(params).map_err(|e| {
            // Mark task as FAILED on execution error
            if let Ok(mut state) = self.state.lock() {
                state.set_task_status(task_id, TaskStatus::Failed);
            }
            A2AError::internal_error(format!("Claude Code execution failed: {}", e))
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
        // Kill the persistent process if one exists
        self.runtime.kill_process(task_id).map_err(|e| {
            A2AError::internal_error(format!("Failed to cancel Claude Code task: {}", e))
        })?;

        // Update state
        {
            let mut state = self.state.lock().map_err(|e| {
                A2AError::internal_error(format!("State lock poisoned: {}", e))
            })?;
            state.set_task_status(task_id, TaskStatus::Canceled);
        }

        log::info!("[ClaudeCodeAdapter] Task {} canceled", task_id);
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
        "claude_code"
    }

    fn runtime_name(&self) -> &str {
        "Claude Code"
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "streaming".to_string(),
            "sessions".to_string(),
            "tool_use".to_string(),
            "structured_output".to_string(),
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
    fn test_claude_adapter_new() {
        let adapter = ClaudeCodeAdapter::new();
        assert_eq!(adapter.runtime_type(), "claude_code");
        assert_eq!(adapter.runtime_name(), "Claude Code");
    }

    #[test]
    fn test_claude_adapter_capabilities() {
        let adapter = ClaudeCodeAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.contains(&"streaming".to_string()));
        assert!(caps.contains(&"tool_use".to_string()));
        assert!(caps.contains(&"sessions".to_string()));
    }

    #[test]
    fn test_claude_adapter_get_task_status_not_found() {
        let adapter = ClaudeCodeAdapter::new();
        let result = adapter.get_task_status("nonexistent");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, -32001); // task_not_found
    }

    #[test]
    fn test_claude_adapter_cancel_task() {
        let adapter = ClaudeCodeAdapter::new();
        // Cancel a non-existent task should succeed (kill_process handles missing)
        let result = adapter.cancel_task("nonexistent-task");
        // kill_process returns Ok even if no process exists
        assert!(result.is_ok());
    }

    #[test]
    fn test_claude_adapter_default() {
        let adapter = ClaudeCodeAdapter::default();
        assert_eq!(adapter.runtime_type(), "claude_code");
    }
}
