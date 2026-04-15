//! Task suggestion parsing and management.
//!
//! Parses `<task-suggestions>` blocks from agent responses,
//! creates task_suggestion messages in channel stores, and
//! provides confirm/dismiss commands for the frontend.

use crate::commands::AppState;
use crate::storage::db_helpers;
use crate::workspace::channel::{self, ChannelMessage, ChannelStore};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single task suggested by an agent during a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedTask {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_priority")]
    pub priority: u32,
    pub assignee: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

fn default_priority() -> u32 {
    3
}

/// Content payload for a task_suggestion message stored in the channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSuggestionContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub suggestions: Vec<SuggestedTask>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmed_task_ids: Option<Vec<String>>,
}

impl TaskSuggestionContent {
    pub fn new(suggestions: Vec<SuggestedTask>) -> Self {
        Self {
            content_type: "task_suggestion".to_string(),
            suggestions,
            status: "pending".to_string(),
            confirmed_task_ids: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Extract `<task-suggestions>...</task-suggestions>` from agent response text.
///
/// Returns an empty Vec if no valid block is found (normal case).
/// Uses lenient parsing: missing fields get defaults, malformed JSON is logged and skipped.
pub fn parse_task_suggestions(response: &str) -> Vec<SuggestedTask> {
    // Find the tag boundaries
    let start_tag = "<task-suggestions>";
    let end_tag = "</task-suggestions>";

    let start_idx = match response.find(start_tag) {
        Some(i) => i + start_tag.len(),
        None => return Vec::new(), // No tag found — normal, agent didn't suggest tasks
    };

    let end_idx = match response.find(end_tag) {
        Some(i) => i,
        None => {
            log::warn!("[task_suggestion] Found opening tag but no closing tag");
            return Vec::new();
        }
    };

    if start_idx >= end_idx {
        return Vec::new();
    }

    let json_str = response[start_idx..end_idx].trim();

    // Parse JSON array
    let raw: Vec<serde_json::Value> = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("[task_suggestion] JSON parse error: {}. Content: {}", e, &json_str[..json_str.len().min(200)]);
            return Vec::new();
        }
    };

    if raw.is_empty() {
        return Vec::new();
    }

    // Parse each item with per-item error tolerance
    let mut suggestions = Vec::new();
    for item in &raw {
        match serde_json::from_value::<SuggestedTask>(item.clone()) {
            Ok(s) => suggestions.push(s),
            Err(e) => {
                log::warn!("[task_suggestion] Failed to parse suggestion item: {}. Item: {}", e, item);
                // Continue with remaining items
            }
        }
    }

    log::info!(
        "[task_suggestion] Parsed {} task suggestions from agent response",
        suggestions.len()
    );

    suggestions
}

// ---------------------------------------------------------------------------
// Message creation helper
// ---------------------------------------------------------------------------

/// Create a task_suggestion message in the channel store.
///
/// Called after parsing suggestions from an agent response.
/// Returns the created message ID.
pub fn create_suggestion_message(
    channel_id: &str,
    agent_id: &str,
    suggestions: Vec<SuggestedTask>,
    channels_dir: &std::path::Path,
) -> Result<String, String> {
    let store = ChannelStore::new(channels_dir);
    let mut ch = store.load(channel_id).map_err(|e| format!("load channel failed: {e}"))?;

    let content = TaskSuggestionContent::new(suggestions);
    let content_json = serde_json::to_string(&content)
        .map_err(|e| format!("serialize suggestion failed: {e}"))?;

    let message_id = crate::workspace::thread::generate_id();
    let msg = ChannelMessage {
        id: message_id.clone(),
        channel_id: channel_id.to_string(),
        sender_type: "agent".to_string(),
        sender_id: agent_id.to_string(),
        content: content_json,
        content_blocks: None,
        timestamp: channel::now_iso(),
    };

    ch.messages.push(msg);
    ch.updated_at = channel::now_iso();
    store.save(&ch).map_err(|e| format!("save channel failed: {e}"))?;

    log::info!(
        "[task_suggestion] Created task_suggestion message {} in channel {} with {} suggestions",
        message_id,
        channel_id,
        content.suggestions.len()
    );

    Ok(message_id)
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

/// Confirm task suggestions — creates Tasks from the selected suggestions.
///
/// For each selected suggestion, creates a Task with source=conversation
/// and binds it to the channel and source message.
#[tauri::command]
pub fn confirm_task_suggestions(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    message_id: String,
    channel_id: String,
    selected: Vec<SuggestedTask>,
) -> Result<Vec<crate::commands::task::TaskInfo>, String> {
    if selected.is_empty() {
        return Err("No tasks selected".to_string());
    }

    let channels_dir = {
        let manager = state
            .agent_manager
            .lock()
            .map_err(|e| format!("lock error: {e}"))?;
        manager.channels_dir()
    };

    // Update the suggestion message status
    let mut confirmed_task_ids = Vec::new();
    {
        let store = ChannelStore::new(&channels_dir);
        let mut ch = store.load(&channel_id).map_err(|e| format!("load failed: {e}"))?;

        if let Some(msg) = ch.messages.iter_mut().find(|m| m.id == message_id) {
            let mut content: TaskSuggestionContent = serde_json::from_str(&msg.content)
                .map_err(|e| format!("parse suggestion content failed: {e}"))?;
            content.status = "confirmed".to_string();
            // Don't set confirmed_task_ids yet — we'll update after creating tasks
            msg.content = serde_json::to_string(&content)
                .map_err(|e| format!("serialize failed: {e}"))?;
        } else {
            return Err(format!("message not found: {message_id}"));
        }

        ch.updated_at = channel::now_iso();
        store.save(&ch).map_err(|e| format!("save failed: {e}"))?;
    }

    // Create tasks for each selected suggestion (directly via db_helpers)
    let mut created_tasks = Vec::new();
    {
        let conn = state
            .db_conn
            .lock()
            .map_err(|e| format!("lock error: {e}"))?;

        for suggestion in &selected {
            let now = db_helpers::chrono_now_iso();
            let task_id = crate::commands::task::generate_task_id_helper();

            let task_row = db_helpers::TaskRow {
                id: task_id.clone(),
                title: suggestion.title.clone(),
                description: suggestion.description.clone(),
                status: "todo".to_string(),
                priority: suggestion.priority as i64,
                creator_type: "agent".to_string(),
                creator_id: "agent".to_string(),
                assignee_id: suggestion.assignee.clone(),
                channel_id: Some(channel_id.clone()),
                thread_id: None,
                parent_task_id: None,
                execution_mode: "realtime".to_string(),
                source: "conversation".to_string(),
                source_message_id: Some(message_id.clone()),
                result: None,
                created_at: now.clone(),
                updated_at: now,
                completed_at: None,
            };

            db_helpers::insert_task(&conn, &task_row)
                .map_err(|e| format!("insert task failed: {e}"))?;

            // Record creation in history
            db_helpers::insert_task_history(
                &conn,
                &task_row.id,
                "status",
                None,
                Some("todo"),
                &format!("agent:agent"),
            ).map_err(|e| format!("insert history failed: {e}"))?;

            let child_count = db_helpers::count_child_tasks(&conn, &task_row.id)
                .unwrap_or(0);
            let dep_count = db_helpers::count_dependencies(&conn, &task_row.id)
                .unwrap_or(0);

            let task_info = crate::commands::task::TaskInfo::from_row(&task_row, child_count, dep_count);
            confirmed_task_ids.push(task_id);
            created_tasks.push(task_info);
        }
    }

    // Update the message with confirmed_task_ids
    {
        let store = ChannelStore::new(&channels_dir);
        let mut ch = store.load(&channel_id).map_err(|e| format!("load failed: {e}"))?;
        if let Some(msg) = ch.messages.iter_mut().find(|m| m.id == message_id) {
            let mut content: TaskSuggestionContent = serde_json::from_str(&msg.content)
                .map_err(|e| format!("parse suggestion content failed: {e}"))?;
            content.confirmed_task_ids = Some(confirmed_task_ids.clone());
            msg.content = serde_json::to_string(&content)
                .map_err(|e| format!("serialize failed: {e}"))?;
        }
        ch.updated_at = channel::now_iso();
        store.save(&ch).map_err(|e| format!("save failed: {e}"))?;
    }

    // Emit task://suggested-confirmed event
    let _ = app.emit(
        "task://suggested-confirmed",
        serde_json::json!({
            "channel_id": channel_id,
            "message_id": message_id,
            "task_ids": confirmed_task_ids,
        }),
    );

    log::info!(
        "[task_suggestion] Confirmed {} tasks from message {} in channel {}",
        created_tasks.len(),
        message_id,
        channel_id
    );

    Ok(created_tasks)
}

/// Dismiss task suggestions — marks the suggestion message as dismissed.
#[tauri::command]
pub fn dismiss_task_suggestions(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    message_id: String,
    channel_id: String,
) -> Result<(), String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let channels_dir = manager.channels_dir();
    let store = ChannelStore::new(&channels_dir);
    let mut ch = store.load(&channel_id).map_err(|e| format!("load failed: {e}"))?;

    if let Some(msg) = ch.messages.iter_mut().find(|m| m.id == message_id) {
        let mut content: TaskSuggestionContent = serde_json::from_str(&msg.content)
            .map_err(|e| format!("parse suggestion content failed: {e}"))?;
        content.status = "dismissed".to_string();
        msg.content = serde_json::to_string(&content)
            .map_err(|e| format!("serialize failed: {e}"))?;
    } else {
        return Err(format!("message not found: {message_id}"));
    }

    ch.updated_at = channel::now_iso();
    store.save(&ch).map_err(|e| format!("save failed: {e}"))?;

    drop(manager);

    // Emit task://suggested-dismissed event
    let _ = app.emit(
        "task://suggested-dismissed",
        serde_json::json!({
            "channel_id": channel_id,
            "message_id": message_id,
        }),
    );

    log::info!(
        "[task_suggestion] Dismissed suggestions in message {} in channel {}",
        message_id,
        channel_id
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_tag() {
        let result = parse_task_suggestions("Hello, this is a normal response without any task suggestions.");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_valid_suggestions() {
        let response = r#"Here's what I think you should do:

<task-suggestions>
[
  {
    "title": "Refactor login module",
    "description": "Break into smaller services",
    "priority": 2,
    "assignee": "Claude"
  },
  {
    "title": "Add unit tests",
    "description": "Cover edge cases"
  }
]
</task-suggestions>

Let me know if you need more details."#;

        let result = parse_task_suggestions(response);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "Refactor login module");
        assert_eq!(result[0].priority, 2);
        assert_eq!(result[0].assignee, Some("Claude".to_string()));
        assert_eq!(result[1].title, "Add unit tests");
        assert_eq!(result[1].priority, 3); // default
        assert_eq!(result[1].assignee, None);
    }

    #[test]
    fn test_parse_empty_array() {
        let response = "<task-suggestions>\n[]\n</task-suggestions>";
        let result = parse_task_suggestions(response);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_invalid_json() {
        let response = "<task-suggestions>\n{invalid json}\n</task-suggestions>";
        let result = parse_task_suggestions(response);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_missing_closing_tag() {
        let response = "<task-suggestions>\n[{\"title\": \"Test\"}]";
        let result = parse_task_suggestions(response);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_partial_items() {
        let response = "<task-suggestions>\n[{\"title\": \"Valid\"}, {\"bad\": true}]\n</task-suggestions>";
        let result = parse_task_suggestions(response);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Valid");
    }

    #[test]
    fn test_suggestion_content_serialization() {
        let content = TaskSuggestionContent::new(vec![
            SuggestedTask {
                title: "Test".to_string(),
                description: "Desc".to_string(),
                priority: 3,
                assignee: None,
                dependencies: vec![],
            },
        ]);
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("\"type\":\"task_suggestion\""));
        assert!(json.contains("\"status\":\"pending\""));

        let parsed: TaskSuggestionContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.content_type, "task_suggestion");
        assert_eq!(parsed.suggestions.len(), 1);
    }
}
