//! CLI-to-A2A adapter trait and shared types.
//!
//! Defines the `CliA2AAdapter` trait that all CLI runtime adapters must implement.
//! Each adapter wraps an existing CLI runtime and exposes it through the A2A
//! protocol's Task lifecycle.

use crate::runtime::StreamEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::super::types::{A2AError, Task, TaskStatus};

// ===========================================================================
// CliA2AAdapter trait
// ===========================================================================

/// Trait for adapting a CLI runtime into an A2A endpoint.
///
/// Implementations wrap an existing `AgentRuntime::execute()` method and
/// manage the mapping between A2A Task lifecycle and CLI process lifecycle.
pub trait CliA2AAdapter: Send + Sync {
    /// Execute a message within an A2A task context.
    ///
    /// The adapter should:
    /// 1. Map the incoming message text to the underlying CLI's input format
    /// 2. Call the underlying runtime's execute method
    /// 3. Return a receiver for streaming events
    /// 4. Track the task status (SUBMITTED -> WORKING -> COMPLETED/FAILED)
    fn execute_task(
        &self,
        task_id: &str,
        message: &str,
        config: &AdapterConfig,
    ) -> Result<std::sync::mpsc::Receiver<StreamEvent>, A2AError>;

    /// Cancel a running task.
    ///
    /// Should kill the underlying CLI process for the given task and
    /// update the task status to CANCELED.
    fn cancel_task(&self, task_id: &str) -> Result<(), A2AError>;

    /// Get the current status of a task.
    fn get_task_status(&self, task_id: &str) -> Result<TaskStatus, A2AError>;

    /// Get the runtime type identifier (e.g. "claude_code", "codex").
    fn runtime_type(&self) -> &str;

    /// Get human-readable runtime name (e.g. "Claude Code", "OpenAI Codex").
    fn runtime_name(&self) -> &str;

    /// List capabilities this adapter supports.
    fn capabilities(&self) -> Vec<String>;

    /// Check if the underlying CLI runtime is available.
    fn is_available(&self) -> bool;
}

// ===========================================================================
// AdapterConfig
// ===========================================================================

/// Configuration for a CLI adapter instance.
///
/// Holds the parameters needed to invoke the underlying CLI runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    /// Workspace directory for CLI execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,

    /// System prompt to prepend/append.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,

    /// Execution timeout in seconds (idle timeout).
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    /// Additional environment variables for the CLI process.
    #[serde(default)]
    pub env_vars: HashMap<String, String>,

    /// Whether to use persistent process (Thread mode) or one-shot (Channel mode).
    #[serde(default)]
    pub persistent: bool,

    /// Optional session ID for resuming conversations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

fn default_timeout_secs() -> u64 {
    300 // 5 minutes
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            workspace: None,
            system_prompt: None,
            timeout_secs: default_timeout_secs(),
            env_vars: HashMap::new(),
            persistent: false,
            session_id: None,
        }
    }
}

impl AdapterConfig {
    /// Create a new config with the given workspace.
    pub fn new(workspace: impl Into<String>) -> Self {
        Self {
            workspace: Some(workspace.into()),
            ..Default::default()
        }
    }

    /// Set system prompt.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set timeout in seconds.
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Enable persistent (Thread) mode.
    pub fn with_persistent(mut self, persistent: bool) -> Self {
        self.persistent = persistent;
        self
    }

    /// Set session ID for conversation resume.
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}

// ===========================================================================
// AdapterState — shared mutable state for tracking active tasks
// ===========================================================================

/// Shared adapter state that tracks active tasks and their statuses.
///
/// This is wrapped in `Arc<Mutex<>>` and shared between the adapter
/// implementation and the A2A server handlers.
#[derive(Debug, Default)]
pub struct AdapterState {
    /// Map of task_id -> current TaskStatus.
    task_statuses: HashMap<String, TaskStatus>,

    /// Map of task_id -> session_id extracted from CLI output.
    task_sessions: HashMap<String, String>,
}

impl AdapterState {
    /// Create a new empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a shared (Arc<Mutex<>>) state.
    pub fn shared() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new()))
    }

    /// Insert or update a task's status.
    pub fn set_task_status(&mut self, task_id: &str, status: TaskStatus) {
        self.task_statuses.insert(task_id.to_string(), status);
    }

    /// Get a task's status.
    pub fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        self.task_statuses.get(task_id).cloned()
    }

    /// Remove a task from tracking.
    pub fn remove_task(&mut self, task_id: &str) {
        self.task_statuses.remove(task_id);
        self.task_sessions.remove(task_id);
    }

    /// Store a session_id for a task.
    pub fn set_session_id(&mut self, task_id: &str, session_id: &str) {
        self.task_sessions
            .insert(task_id.to_string(), session_id.to_string());
    }

    /// Get the session_id for a task.
    pub fn get_session_id(&self, task_id: &str) -> Option<String> {
        self.task_sessions.get(task_id).cloned()
    }

    /// List all tracked task IDs.
    pub fn task_ids(&self) -> Vec<String> {
        self.task_statuses.keys().cloned().collect()
    }

    /// Build an A2A Task from tracked state.
    pub fn build_task(&self, task_id: &str) -> Option<Task> {
        self.task_statuses.get(task_id).map(|status| Task {
            id: task_id.to_string(),
            status: status.clone(),
            session_id: self.task_sessions.get(task_id).cloned(),
            messages: Vec::new(),
            artifacts: Vec::new(),
            metadata: None,
        })
    }

    /// Build all tracked tasks.
    pub fn build_all_tasks(&self) -> Vec<Task> {
        self.task_statuses
            .keys()
            .filter_map(|id| self.build_task(id))
            .collect()
    }
}

// ===========================================================================
// Helper: spawn a status-tracking event reader
// ===========================================================================

/// Reads events from a StreamEvent receiver and updates task status in shared state.
///
/// This function spawns a background thread that:
/// 1. Reads streaming events from the CLI runtime
/// 2. Updates the shared AdapterState with task status transitions
/// 3. Forwards events to the caller's receiver
///
/// Returns a new receiver that the caller can read from.
pub fn spawn_status_tracker(
    task_id: String,
    rx: std::sync::mpsc::Receiver<StreamEvent>,
    state: Arc<Mutex<AdapterState>>,
) -> std::sync::mpsc::Receiver<StreamEvent> {
    let (tx, caller_rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        for event in rx.iter() {
            // Track session_id if present
            if let Some(ref sid) = event.session_id {
                if let Ok(mut s) = state.lock() {
                    s.set_session_id(&task_id, sid);
                }
            }

            // Update task status based on event type
            if let Ok(mut s) = state.lock() {
                if event.is_done {
                    if event.error.is_some() {
                        s.set_task_status(&task_id, TaskStatus::Failed);
                    } else {
                        s.set_task_status(&task_id, TaskStatus::Completed);
                    }
                }
            }

            // Forward event to caller
            if tx.send(event).is_err() {
                // Caller dropped the receiver, stop tracking
                break;
            }
        }
    });

    caller_rx
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_config_default() {
        let config = AdapterConfig::default();
        assert!(config.workspace.is_none());
        assert!(config.system_prompt.is_none());
        assert_eq!(config.timeout_secs, 300);
        assert!(!config.persistent);
        assert!(config.session_id.is_none());
    }

    #[test]
    fn test_adapter_config_builder() {
        let config = AdapterConfig::new("/workspace")
            .with_system_prompt("You are helpful")
            .with_timeout(60)
            .with_persistent(true)
            .with_session_id("sess-1");

        assert_eq!(config.workspace.as_deref(), Some("/workspace"));
        assert_eq!(config.system_prompt.as_deref(), Some("You are helpful"));
        assert_eq!(config.timeout_secs, 60);
        assert!(config.persistent);
        assert_eq!(config.session_id.as_deref(), Some("sess-1"));
    }

    #[test]
    fn test_adapter_state_task_tracking() {
        let state = AdapterState::new();
        let task_id = "task-001";

        // Initially no task
        assert!(state.get_task_status(task_id).is_none());
        assert!(state.build_task(task_id).is_none());

        // Add task
        let mut state = state;
        state.set_task_status(task_id, TaskStatus::Working);
        assert_eq!(state.get_task_status(task_id), Some(TaskStatus::Working));

        // Build task
        let task = state.build_task(task_id).unwrap();
        assert_eq!(task.id, task_id);
        assert_eq!(task.status, TaskStatus::Working);
    }

    #[test]
    fn test_adapter_state_session_tracking() {
        let mut state = AdapterState::new();
        state.set_task_status("t-1", TaskStatus::Working);
        state.set_session_id("t-1", "sess-abc");

        assert_eq!(state.get_session_id("t-1"), Some("sess-abc".to_string()));

        let task = state.build_task("t-1").unwrap();
        assert_eq!(task.session_id, Some("sess-abc".to_string()));
    }

    #[test]
    fn test_adapter_state_remove_task() {
        let mut state = AdapterState::new();
        state.set_task_status("t-1", TaskStatus::Completed);
        state.set_session_id("t-1", "sess-1");
        state.remove_task("t-1");

        assert!(state.get_task_status("t-1").is_none());
        assert!(state.get_session_id("t-1").is_none());
    }

    #[test]
    fn test_adapter_state_build_all_tasks() {
        let mut state = AdapterState::new();
        state.set_task_status("t-1", TaskStatus::Working);
        state.set_task_status("t-2", TaskStatus::Completed);

        let tasks = state.build_all_tasks();
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn test_adapter_state_shared() {
        let state = AdapterState::shared();
        {
            let mut s = state.lock().unwrap();
            s.set_task_status("t-1", TaskStatus::Submitted);
        }
        let s = state.lock().unwrap();
        assert_eq!(s.get_task_status("t-1"), Some(TaskStatus::Submitted));
    }

    #[test]
    fn test_spawn_status_tracker() {
        let state = AdapterState::shared();
        let (tx, rx) = std::sync::mpsc::channel();

        let caller_rx = spawn_status_tracker("t-track".to_string(), rx, state.clone());

        // Send some events
        tx.send(StreamEvent {
            text: "working".into(),
            is_done: false,
            error: None,
            msg_type: Some("assistant".into()),
            session_id: Some("sess-tracker".into()),
            content_blocks: None,
            token_usage: None,
        }).unwrap();

        tx.send(StreamEvent {
            text: "done".into(),
            is_done: true,
            error: None,
            msg_type: Some("result".into()),
            session_id: None,
            content_blocks: None,
            token_usage: None,
        }).unwrap();

        drop(tx);

        // Read forwarded events
        let events: Vec<_> = caller_rx.iter().collect();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].text, "working");
        assert_eq!(events[1].text, "done");

        // Check state was updated
        let s = state.lock().unwrap();
        assert_eq!(s.get_task_status("t-track"), Some(TaskStatus::Completed));
        assert_eq!(s.get_session_id("t-track"), Some("sess-tracker".into()));
    }
}
