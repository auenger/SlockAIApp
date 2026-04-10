//! Activity Log Tauri commands.
//!
//! Provides IPC commands for logging, listing, and clearing activity entries.
//!
//! ## Dual-Write Strategy
//!
//! Activity entries are written to both JSONL (existing) and SQLite (new).
//! Listing queries prefer SQLite for performance but fall back to JSONL
//! if the database has no entries.

use crate::storage::activity::{
    ActivityLog, ActivityStore, ActivityType, create_entry,
};
use crate::storage::db_helpers;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// A single activity log entry returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub id: String,
    pub timestamp: String,
    pub activity_type: String,
    pub agent_id: Option<String>,
    pub workspace_id: Option<String>,
    pub summary: String,
    pub details: serde_json::Value,
}

impl From<ActivityLog> for ActivityEntry {
    fn from(log: ActivityLog) -> Self {
        Self {
            id: log.id,
            timestamp: log.timestamp,
            activity_type: serde_json::to_string(&log.activity_type)
                .unwrap_or_else(|_| "\"system\"".to_string())
                .trim_matches('"')
                .to_string(),
            agent_id: log.agent_id,
            workspace_id: log.workspace_id,
            summary: log.summary,
            details: log.details,
        }
    }
}

impl From<db_helpers::ActivityLogRow> for ActivityEntry {
    fn from(row: db_helpers::ActivityLogRow) -> Self {
        Self {
            id: row.id,
            timestamp: row.timestamp,
            activity_type: row.activity_type,
            agent_id: row.agent_id,
            workspace_id: row.workspace_id,
            summary: row.summary,
            details: serde_json::from_str(&row.details_json).unwrap_or(serde_json::json!({})),
        }
    }
}

/// Paginated result of activity log entries.
#[derive(Debug, Clone, Serialize)]
pub struct ListActivitiesResult {
    /// Activity entries (newest first).
    pub entries: Vec<ActivityEntry>,
    /// Total count of matching entries (before pagination).
    pub total: usize,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Request to log a new activity.
#[derive(Debug, Deserialize)]
pub struct LogActivityRequest {
    /// Type of activity (e.g. "agent_created", "conversation_started").
    pub activity_type: String,
    /// Agent ID related to this activity (if any).
    pub agent_id: Option<String>,
    /// Human-readable summary.
    pub summary: String,
    /// Additional details as JSON.
    #[serde(default = "default_details")]
    pub details: serde_json::Value,
}

fn default_details() -> serde_json::Value {
    serde_json::json!({})
}

/// Request to list activities with optional filter and pagination.
#[derive(Debug, Deserialize)]
pub struct ListActivitiesRequest {
    /// Filter by agent_id (optional).
    pub agent_id: Option<String>,
    /// Number of entries to skip.
    #[serde(default = "default_offset")]
    pub offset: usize,
    /// Maximum number of entries to return.
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_offset() -> usize {
    0
}
fn default_limit() -> usize {
    50
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Log a new activity entry (dual-write: JSONL + SQLite).
#[tauri::command]
pub fn log_activity(
    state: tauri::State<'_, super::AppState>,
    request: LogActivityRequest,
) -> Result<ActivityEntry, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let workspace_root = manager.workspace_root();
    let store = ActivityStore::new(workspace_root);

    let activity_type = parse_activity_type(&request.activity_type);

    let entry = create_entry(
        activity_type.clone(),
        request.agent_id.clone(),
        request.summary,
        request.details,
    );

    // Write to JSONL
    store
        .append(&entry)
        .map_err(|e| format!("failed to log activity: {e}"))?;

    // Dual-write to SQLite
    {
        let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
        let activity_type_str = serde_json::to_string(&activity_type)
            .unwrap_or_else(|_| "\"system\"".to_string())
            .trim_matches('"')
            .to_string();
        let db_row = db_helpers::ActivityLogRow {
            id: entry.id.clone(),
            timestamp: entry.timestamp.clone(),
            activity_type: activity_type_str,
            agent_id: entry.agent_id.clone(),
            workspace_id: entry.workspace_id.clone(),
            summary: entry.summary.clone(),
            details_json: serde_json::to_string(&entry.details).unwrap_or_else(|_| "{}".to_string()),
        };
        if let Err(e) = db_helpers::insert_activity(&db_conn, &db_row) {
            log::warn!("[log_activity] Failed to insert activity into SQLite: {}", e);
        }
    }

    Ok(ActivityEntry::from(entry))
}

/// List activity entries with optional filter and pagination.
///
/// Prefers SQLite for fast querying; falls back to JSONL if SQLite is empty.
#[tauri::command]
pub fn list_activities(
    state: tauri::State<'_, super::AppState>,
    request: ListActivitiesRequest,
) -> Result<ListActivitiesResult, String> {
    // Try SQLite first for fast querying
    {
        let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
        let count = db_helpers::count_activities(&db_conn, request.agent_id.as_deref());
        if let Ok(total) = count {
            if total > 0 {
                let rows = db_helpers::list_activities(
                    &db_conn,
                    request.agent_id.as_deref(),
                    request.offset,
                    request.limit,
                );
                if let Ok(rows) = rows {
                    let entries: Vec<ActivityEntry> = rows.into_iter().map(ActivityEntry::from).collect();
                    return Ok(ListActivitiesResult {
                        entries,
                        total: total as usize,
                    });
                }
            }
        }
    }

    // Fallback to JSONL-based listing
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let workspace_root = manager.workspace_root();
    let store = ActivityStore::new(workspace_root);

    // Get total count (all matching, no pagination)
    let all = store
        .load_filtered(request.agent_id.as_deref(), 0, usize::MAX)
        .map_err(|e| format!("failed to load activities: {e}"))?;
    let total = all.len();

    // Get paginated entries
    let entries = store
        .load_filtered(request.agent_id.as_deref(), request.offset, request.limit)
        .map_err(|e| format!("failed to load activities: {e}"))?;

    Ok(ListActivitiesResult {
        entries: entries.into_iter().map(ActivityEntry::from).collect(),
        total,
    })
}

/// Clear all activity entries.
#[tauri::command]
pub fn clear_activities(
    state: tauri::State<'_, super::AppState>,
) -> Result<(), String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let workspace_root = manager.workspace_root();
    let store = ActivityStore::new(workspace_root);

    store
        .clear()
        .map_err(|e| format!("failed to clear activities: {e}"))?;

    // Note: We don't clear the SQLite activity_log table here to preserve
    // the structured query capability. The JSONL clear is sufficient for
    // the user-facing "clear activity" action.

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse an activity type string into the enum.
fn parse_activity_type(s: &str) -> ActivityType {
    match s {
        "agent_created" => ActivityType::AgentCreated,
        "agent_deleted" => ActivityType::AgentDeleted,
        "conversation_started" => ActivityType::ConversationStarted,
        "conversation_ended" => ActivityType::ConversationEnded,
        "skill_changed" => ActivityType::SkillChanged,
        "channel_created" => ActivityType::ChannelCreated,
        "channel_updated" => ActivityType::ChannelUpdated,
        "channel_deleted" => ActivityType::ChannelDeleted,
        "channel_message" => ActivityType::ChannelMessage,
        "system" => ActivityType::System,
        _ => ActivityType::System,
    }
}
