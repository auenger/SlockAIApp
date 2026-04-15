//! Task Delegation engine for A2A multi-agent collaboration.
//!
//! Manages the lifecycle of task delegations from one Agent to another.
//! A delegation represents Agent A asking Agent B to execute a sub-task,
//! with context summary, status tracking, and result handling.
//!
//! ## Connection-Centric Design
//!
//! Delegation targets are resolved via the ConnectionMode model:
//! - Local agents communicate via A2A Unix Socket (A2A Server adapter).
//! - Remote agents communicate via A2A HTTPS (RemoteA2ARuntime).
//!
//! The delegation engine is agnostic to the transport layer.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::types::{ConnectionMode, Message, Task, TaskStatus};

// ===========================================================================
// Delegation Status
// ===========================================================================

/// Status of a delegation request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DelegationStatus {
    /// Delegation created but not yet sent.
    #[default]
    Pending,
    /// Delegation sent to the target agent.
    Sent,
    /// Target agent acknowledged the delegation.
    Acknowledged,
    /// Target agent is working on the task.
    InProgress,
    /// Delegation completed successfully.
    Completed,
    /// Delegation failed.
    Failed,
    /// Delegation was cancelled by the source agent.
    Cancelled,
    /// Delegation timed out waiting for response.
    TimedOut,
}

impl DelegationStatus {
    /// Whether this status is terminal (no further transitions).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::TimedOut
        )
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Sent => "SENT",
            Self::Acknowledged => "ACKNOWLEDGED",
            Self::InProgress => "IN_PROGRESS",
            Self::Completed => "COMPLETED",
            Self::Failed => "FAILED",
            Self::Cancelled => "CANCELLED",
            Self::TimedOut => "TIMED_OUT",
        }
    }
}

// ===========================================================================
// Delegation Request
// ===========================================================================

/// A request to delegate a task from one agent to another.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DelegationRequest {
    /// Unique delegation identifier.
    pub id: String,
    /// The agent initiating the delegation.
    pub from_agent_id: String,
    /// The target agent to delegate to.
    pub to_agent_id: String,
    /// Description of the task to delegate.
    pub task_description: String,
    /// Auto-extracted context summary from the current conversation.
    pub context_summary: String,
    /// Optional parent task ID for tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_task_id: Option<String>,
    /// Optional channel ID where this delegation originated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// Current status of the delegation.
    #[serde(default)]
    pub status: DelegationStatus,
    /// Connection mode for the target agent (resolved at creation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_connection_mode: Option<ConnectionMode>,
    /// Result from the target agent (set on completion).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// Error message if the delegation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 last update timestamp.
    pub updated_at: String,
}

// ===========================================================================
// Delegation Manager
// ===========================================================================

/// Manages task delegations between agents.
///
/// Thread-safe via Arc<Mutex<HashMap>>.
pub struct DelegationManager {
    /// Active and recent delegations indexed by ID.
    delegations: Arc<Mutex<HashMap<String, DelegationRequest>>>,
}

impl DelegationManager {
    /// Create a new DelegationManager.
    pub fn new() -> Self {
        Self {
            delegations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new delegation request.
    ///
    /// Generates a unique ID and initializes the status to Pending.
    pub fn create(
        &self,
        from_agent_id: &str,
        to_agent_id: &str,
        task_description: &str,
        context_summary: &str,
        parent_task_id: Option<&str>,
        channel_id: Option<&str>,
        target_connection_mode: Option<ConnectionMode>,
    ) -> Result<DelegationRequest, String> {
        let id = generate_delegation_id();
        let now = now_iso();

        let delegation = DelegationRequest {
            id: id.clone(),
            from_agent_id: from_agent_id.to_string(),
            to_agent_id: to_agent_id.to_string(),
            task_description: task_description.to_string(),
            context_summary: context_summary.to_string(),
            parent_task_id: parent_task_id.map(|s| s.to_string()),
            channel_id: channel_id.map(|s| s.to_string()),
            status: DelegationStatus::Pending,
            target_connection_mode,
            result: None,
            error: None,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut delegations = self.delegations.lock().map_err(|e| e.to_string())?;
        delegations.insert(id, delegation.clone());

        log::info!(
            "[DelegationManager] Created delegation {} ({} → {}): {}",
            delegation.id,
            from_agent_id,
            to_agent_id,
            &task_description[..task_description.len().min(80)]
        );

        Ok(delegation)
    }

    /// Update the status of a delegation.
    pub fn update_status(
        &self,
        delegation_id: &str,
        new_status: DelegationStatus,
    ) -> Result<DelegationRequest, String> {
        let mut delegations = self.delegations.lock().map_err(|e| e.to_string())?;

        let delegation = delegations
            .get_mut(delegation_id)
            .ok_or_else(|| format!("Delegation not found: {}", delegation_id))?;

        if delegation.status.is_terminal() {
            return Err(format!(
                "Cannot update terminal delegation {} (current: {})",
                delegation_id,
                delegation.status.as_str()
            ));
        }

        log::info!(
            "[DelegationManager] Status update: {} → {} for delegation {}",
            delegation.status.as_str(),
            new_status.as_str(),
            delegation_id
        );

        delegation.status = new_status;
        delegation.updated_at = now_iso();

        Ok(delegation.clone())
    }

    /// Set the result of a completed delegation.
    pub fn set_result(
        &self,
        delegation_id: &str,
        result: &str,
    ) -> Result<DelegationRequest, String> {
        let mut delegations = self.delegations.lock().map_err(|e| e.to_string())?;

        let delegation = delegations
            .get_mut(delegation_id)
            .ok_or_else(|| format!("Delegation not found: {}", delegation_id))?;

        delegation.status = DelegationStatus::Completed;
        delegation.result = Some(result.to_string());
        delegation.updated_at = now_iso();

        log::info!(
            "[DelegationManager] Delegation {} completed with result ({} chars)",
            delegation_id,
            result.len()
        );

        Ok(delegation.clone())
    }

    /// Set the error for a failed delegation.
    pub fn set_error(
        &self,
        delegation_id: &str,
        error: &str,
    ) -> Result<DelegationRequest, String> {
        let mut delegations = self.delegations.lock().map_err(|e| e.to_string())?;

        let delegation = delegations
            .get_mut(delegation_id)
            .ok_or_else(|| format!("Delegation not found: {}", delegation_id))?;

        delegation.status = DelegationStatus::Failed;
        delegation.error = Some(error.to_string());
        delegation.updated_at = now_iso();

        log::warn!(
            "[DelegationManager] Delegation {} failed: {}",
            delegation_id,
            error
        );

        Ok(delegation.clone())
    }

    /// Cancel a delegation.
    pub fn cancel(&self, delegation_id: &str) -> Result<DelegationRequest, String> {
        let mut delegations = self.delegations.lock().map_err(|e| e.to_string())?;

        let delegation = delegations
            .get_mut(delegation_id)
            .ok_or_else(|| format!("Delegation not found: {}", delegation_id))?;

        if delegation.status.is_terminal() {
            return Err(format!(
                "Cannot cancel terminal delegation {} (current: {})",
                delegation_id,
                delegation.status.as_str()
            ));
        }

        delegation.status = DelegationStatus::Cancelled;
        delegation.updated_at = now_iso();

        log::info!(
            "[DelegationManager] Delegation {} cancelled",
            delegation_id
        );

        Ok(delegation.clone())
    }

    /// Get a delegation by ID.
    pub fn get(&self, delegation_id: &str) -> Result<Option<DelegationRequest>, String> {
        let delegations = self.delegations.lock().map_err(|e| e.to_string())?;
        Ok(delegations.get(delegation_id).cloned())
    }

    /// List all delegations.
    pub fn list_all(&self) -> Result<Vec<DelegationRequest>, String> {
        let delegations = self.delegations.lock().map_err(|e| e.to_string())?;
        let mut result: Vec<_> = delegations.values().cloned().collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }

    /// List delegations for a specific source agent.
    pub fn list_by_from_agent(&self, agent_id: &str) -> Result<Vec<DelegationRequest>, String> {
        let delegations = self.delegations.lock().map_err(|e| e.to_string())?;
        let mut result: Vec<_> = delegations
            .values()
            .filter(|d| d.from_agent_id == agent_id)
            .cloned()
            .collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }

    /// List delegations targeting a specific agent.
    pub fn list_by_to_agent(&self, agent_id: &str) -> Result<Vec<DelegationRequest>, String> {
        let delegations = self.delegations.lock().map_err(|e| e.to_string())?;
        let mut result: Vec<_> = delegations
            .values()
            .filter(|d| d.to_agent_id == agent_id)
            .cloned()
            .collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }

    /// List active (non-terminal) delegations.
    pub fn list_active(&self) -> Result<Vec<DelegationRequest>, String> {
        let delegations = self.delegations.lock().map_err(|e| e.to_string())?;
        let result: Vec<_> = delegations
            .values()
            .filter(|d| !d.status.is_terminal())
            .cloned()
            .collect();
        Ok(result)
    }

    /// Build the delegation message to send to the target agent.
    ///
    /// Constructs a user message containing the context summary and
    /// task description, formatted for the target agent.
    pub fn build_delegation_message(delegation: &DelegationRequest) -> Message {
        let mut parts_text = format!(
            "[Delegated Task from Agent {}]\n\n## Task\n{}\n\n## Context\n{}",
            delegation.from_agent_id,
            delegation.task_description,
            delegation.context_summary
        );

        if let Some(ref parent_task) = delegation.parent_task_id {
            parts_text.push_str(&format!("\n\n**Parent Task:** {}", parent_task));
        }

        Message::user_text(parts_text)
    }
}

impl Default for DelegationManager {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Context Summary Extraction
// ===========================================================================

/// Extract a context summary from a slice of conversation messages.
///
/// Takes the last N messages and produces a compact summary suitable
/// for including in a delegation request.
pub fn extract_context_summary(
    messages: &[crate::workspace::channel::ChannelMessage],
    max_messages: usize,
) -> String {
    let recent: Vec<_> = messages.iter().rev().take(max_messages).collect();
    let mut summary_parts = Vec::new();

    for msg in recent.iter().rev() {
        let sender = if msg.sender_type == "user" {
            "User".to_string()
        } else {
            msg.sender_id.clone()
        };

        // Truncate long messages for summary
        let content = if msg.content.len() > 500 {
            format!("{}...", &msg.content[..500])
        } else {
            msg.content.clone()
        };

        summary_parts.push(format!("[{}]: {}", sender, content));
    }

    if summary_parts.is_empty() {
        "No prior context available.".to_string()
    } else {
        summary_parts.join("\n")
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Generate a unique delegation ID.
fn generate_delegation_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("del-{:x}", nanos)
}

/// Get current ISO 8601 timestamp.
fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;
    let (y, m, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, hours, minutes, seconds)
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let diy = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 { 366 } else { 365 };
        if days < diy { break; }
        days -= diy;
        year += 1;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let md: [u64; 12] = if leap { [31,29,31,30,31,30,31,31,30,31,30,31] } else { [31,28,31,30,31,30,31,31,30,31,30,31] };
    let mut month = 1u64;
    for &x in &md { if days < x { break; } days -= x; month += 1; }
    (year, month, days + 1)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegation_status_terminal() {
        assert!(!DelegationStatus::Pending.is_terminal());
        assert!(!DelegationStatus::Sent.is_terminal());
        assert!(!DelegationStatus::InProgress.is_terminal());
        assert!(DelegationStatus::Completed.is_terminal());
        assert!(DelegationStatus::Failed.is_terminal());
        assert!(DelegationStatus::Cancelled.is_terminal());
        assert!(DelegationStatus::TimedOut.is_terminal());
    }

    #[test]
    fn test_delegation_status_serde() {
        let status = DelegationStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"IN_PROGRESS\"");
        let back: DelegationStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, back);
    }

    #[test]
    fn test_delegation_request_serde() {
        let req = DelegationRequest {
            id: "del-1".into(),
            from_agent_id: "agent-a".into(),
            to_agent_id: "agent-b".into(),
            task_description: "Write unit tests".into(),
            context_summary: "Working on auth module".into(),
            parent_task_id: Some("task-1".into()),
            channel_id: Some("ch-1".into()),
            status: DelegationStatus::Pending,
            target_connection_mode: Some(ConnectionMode::Local),
            result: None,
            error: None,
            created_at: "2026-04-16T12:00:00Z".into(),
            updated_at: "2026-04-16T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: DelegationRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, back);
    }

    #[test]
    fn test_manager_create() {
        let manager = DelegationManager::new();
        let delegation = manager.create(
            "agent-a",
            "agent-b",
            "Write tests",
            "Context here",
            None,
            None,
            Some(ConnectionMode::Local),
        ).unwrap();

        assert_eq!(delegation.from_agent_id, "agent-a");
        assert_eq!(delegation.to_agent_id, "agent-b");
        assert_eq!(delegation.status, DelegationStatus::Pending);
    }

    #[test]
    fn test_manager_status_lifecycle() {
        let manager = DelegationManager::new();
        let delegation = manager.create(
            "agent-a", "agent-b", "Do work", "ctx", None, None, None,
        ).unwrap();

        let id = delegation.id.clone();

        // Pending → Sent
        let d = manager.update_status(&id, DelegationStatus::Sent).unwrap();
        assert_eq!(d.status, DelegationStatus::Sent);

        // Sent → Acknowledged
        let d = manager.update_status(&id, DelegationStatus::Acknowledged).unwrap();
        assert_eq!(d.status, DelegationStatus::Acknowledged);

        // Acknowledged → InProgress
        let d = manager.update_status(&id, DelegationStatus::InProgress).unwrap();
        assert_eq!(d.status, DelegationStatus::InProgress);

        // InProgress → Completed (via set_result)
        let d = manager.set_result(&id, "Task done!").unwrap();
        assert_eq!(d.status, DelegationStatus::Completed);
        assert_eq!(d.result, Some("Task done!".to_string()));

        // Cannot update terminal status
        let result = manager.update_status(&id, DelegationStatus::Sent);
        assert!(result.is_err());
    }

    #[test]
    fn test_manager_cancel() {
        let manager = DelegationManager::new();
        let delegation = manager.create(
            "agent-a", "agent-b", "Do work", "ctx", None, None, None,
        ).unwrap();
        let id = delegation.id;

        let d = manager.cancel(&id).unwrap();
        assert_eq!(d.status, DelegationStatus::Cancelled);

        // Cannot cancel again
        assert!(manager.cancel(&id).is_err());
    }

    #[test]
    fn test_manager_error() {
        let manager = DelegationManager::new();
        let delegation = manager.create(
            "agent-a", "agent-b", "Do work", "ctx", None, None, None,
        ).unwrap();
        let id = delegation.id;

        let d = manager.set_error(&id, "Agent unreachable").unwrap();
        assert_eq!(d.status, DelegationStatus::Failed);
        assert_eq!(d.error, Some("Agent unreachable".to_string()));
    }

    #[test]
    fn test_manager_list_by_agent() {
        let manager = DelegationManager::new();

        manager.create("agent-a", "agent-b", "Task 1", "ctx", None, None, None).unwrap();
        manager.create("agent-a", "agent-c", "Task 2", "ctx", None, None, None).unwrap();
        manager.create("agent-b", "agent-a", "Task 3", "ctx", None, None, None).unwrap();

        let from_a = manager.list_by_from_agent("agent-a").unwrap();
        assert_eq!(from_a.len(), 2);

        let to_a = manager.list_by_to_agent("agent-a").unwrap();
        assert_eq!(to_a.len(), 1);
    }

    #[test]
    fn test_manager_list_active() {
        let manager = DelegationManager::new();

        let d1 = manager.create("agent-a", "agent-b", "Task 1", "ctx", None, None, None).unwrap();
        let d2 = manager.create("agent-a", "agent-c", "Task 2", "ctx", None, None, None).unwrap();

        // Complete one
        manager.set_result(&d1.id, "Done").unwrap();

        let active = manager.list_active().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, d2.id);
    }

    #[test]
    fn test_build_delegation_message() {
        let delegation = DelegationRequest {
            id: "del-1".into(),
            from_agent_id: "agent-a".into(),
            to_agent_id: "agent-b".into(),
            task_description: "Write unit tests".into(),
            context_summary: "We are working on the auth module".into(),
            parent_task_id: Some("task-1".into()),
            channel_id: None,
            status: DelegationStatus::Pending,
            target_connection_mode: None,
            result: None,
            error: None,
            created_at: "2026-04-16T12:00:00Z".into(),
            updated_at: "2026-04-16T12:00:00Z".into(),
        };

        let msg = DelegationManager::build_delegation_message(&delegation);
        assert_eq!(msg.role, super::super::types::MessageRole::User);
        // Check the text content in the first Part
        match &msg.parts[0] {
            super::super::types::Part::Text { text } => {
                assert!(text.contains("agent-a"));
                assert!(text.contains("Write unit tests"));
                assert!(text.contains("Parent Task"));
            }
            _ => panic!("Expected Text part"),
        }
    }

    #[test]
    fn test_extract_context_summary() {
        let messages = vec![
            crate::workspace::channel::ChannelMessage {
                id: "m1".into(),
                channel_id: "ch-1".into(),
                sender_type: "user".into(),
                sender_id: "user".into(),
                content: "Hello agent".into(),
                content_blocks: None,
                timestamp: "2026-04-16T12:00:00Z".into(),
            },
            crate::workspace::channel::ChannelMessage {
                id: "m2".into(),
                channel_id: "ch-1".into(),
                sender_type: "agent".into(),
                sender_id: "agent-a".into(),
                content: "Hi! How can I help?".into(),
                content_blocks: None,
                timestamp: "2026-04-16T12:00:01Z".into(),
            },
        ];

        let summary = extract_context_summary(&messages, 10);
        assert!(summary.contains("[User]: Hello agent"));
        assert!(summary.contains("[agent-a]: Hi! How can I help?"));
    }

    #[test]
    fn test_extract_context_summary_truncation() {
        let long_content: String = "x".repeat(600);
        let messages = vec![
            crate::workspace::channel::ChannelMessage {
                id: "m1".into(),
                channel_id: "ch-1".into(),
                sender_type: "user".into(),
                sender_id: "user".into(),
                content: long_content,
                content_blocks: None,
                timestamp: "2026-04-16T12:00:00Z".into(),
            },
        ];

        let summary = extract_context_summary(&messages, 10);
        assert!(summary.contains("..."));
        assert!(summary.len() < 600);
    }
}
