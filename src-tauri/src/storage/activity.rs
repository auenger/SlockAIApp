//! Activity Log storage engine.
//!
//! Records agent activities (creation, deletion, conversation events, etc.)
//! in a JSONL file for persistent timeline display.
//!
//! File path pattern:
//!   `{workspace_root}/activity.jsonl`
//!
//! Each line is a JSON object representing a single activity event.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ===========================================================================
// Activity Log Data Model
// ===========================================================================

/// Type of activity event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    /// Agent created.
    AgentCreated,
    /// Agent deleted.
    AgentDeleted,
    /// Conversation (thread) started.
    ConversationStarted,
    /// Conversation ended / thread deleted.
    ConversationEnded,
    /// Skill enabled or disabled.
    SkillChanged,
    /// Channel created.
    ChannelCreated,
    /// Channel updated.
    ChannelUpdated,
    /// Channel deleted.
    ChannelDeleted,
    /// Message sent in a channel.
    ChannelMessage,
    /// Generic system event.
    System,
}

/// A single activity log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    /// Unique identifier for this activity entry.
    pub id: String,
    /// ISO 8601 timestamp of the activity.
    pub timestamp: String,
    /// Type of activity.
    pub activity_type: ActivityType,
    /// The agent this activity relates to (if applicable).
    pub agent_id: Option<String>,
    /// The workspace ID (currently the workspace root hash or identifier).
    pub workspace_id: Option<String>,
    /// Human-readable summary of the activity.
    pub summary: String,
    /// Additional details as key-value pairs.
    #[serde(default)]
    pub details: serde_json::Value,
}

// ===========================================================================
// Activity Store
// ===========================================================================

/// Append-only JSONL storage for activity logs.
///
/// All activities are written to a single `activity.jsonl` file in the
/// workspace root. Entries are appended in chronological order.
pub struct ActivityStore {
    /// Path to the activity JSONL file.
    file_path: PathBuf,
}

impl ActivityStore {
    /// Create a new ActivityStore rooted at the given workspace directory.
    ///
    /// The activity file will be at `{workspace_root}/activity.jsonl`.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root = workspace_root.into();
        Self {
            file_path: root.join("activity.jsonl"),
        }
    }

    /// Ensure the parent directory exists.
    fn ensure_dir(&self) -> Result<(), ActivityError> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| ActivityError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        Ok(())
    }

    /// Append a single activity log entry.
    pub fn append(&self, entry: &ActivityLog) -> Result<(), ActivityError> {
        self.ensure_dir()?;

        let mut line = serde_json::to_string(entry).map_err(|e| ActivityError::Serialization {
            message: e.to_string(),
        })?;
        line.push('\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .map_err(|e| ActivityError::Io {
                path: self.file_path.clone(),
                source: e,
            })?;

        file.write_all(line.as_bytes())
            .map_err(|e| ActivityError::Io {
                path: self.file_path.clone(),
                source: e,
            })?;

        file.flush().map_err(|e| ActivityError::Io {
            path: self.file_path.clone(),
            source: e,
        })?;

        Ok(())
    }

    /// Load all activity entries.
    ///
    /// Returns entries in chronological order (oldest first).
    /// Malformed lines are skipped with a warning log.
    pub fn load_all(&self) -> Result<Vec<ActivityLog>, ActivityError> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.file_path).map_err(|e| ActivityError::Io {
            path: self.file_path.clone(),
            source: e,
        })?;

        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = match line_result {
                Ok(l) => l,
                Err(e) => {
                    log::warn!(
                        "[ActivityStore] Failed to read line {}: {}",
                        line_num + 1,
                        e
                    );
                    continue;
                }
            };

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            match serde_json::from_str::<ActivityLog>(trimmed) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    log::warn!(
                        "[ActivityStore] Failed to parse line {}: {}",
                        line_num + 1,
                        e
                    );
                    continue;
                }
            }
        }

        Ok(entries)
    }

    /// Load activities with pagination and optional agent filter.
    ///
    /// Returns entries in reverse chronological order (newest first).
    /// `offset` is the number of entries to skip (after filtering).
    /// `limit` is the maximum number of entries to return.
    pub fn load_filtered(
        &self,
        agent_id: Option<&str>,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<ActivityLog>, ActivityError> {
        let mut entries = self.load_all()?;

        // Filter by agent_id if specified
        if let Some(aid) = agent_id {
            entries.retain(|e| e.agent_id.as_deref() == Some(aid));
        }

        // Reverse to get newest first
        entries.reverse();

        // Apply pagination
        let entries = entries.into_iter().skip(offset).take(limit).collect();

        Ok(entries)
    }

    /// Clear all activity entries (delete the file).
    pub fn clear(&self) -> Result<(), ActivityError> {
        if self.file_path.exists() {
            fs::remove_file(&self.file_path).map_err(|e| ActivityError::Io {
                path: self.file_path.clone(),
                source: e,
            })?;
        }
        Ok(())
    }
}

// ===========================================================================
// Error type
// ===========================================================================

/// Activity storage errors.
#[derive(Debug, thiserror::Error)]
pub enum ActivityError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("serialization error: {message}")]
    Serialization { message: String },
}

// ===========================================================================
// Helper for creating activity entries
// ===========================================================================

/// Helper to generate a new unique ID for an activity entry.
pub fn generate_activity_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("act-{:x}-{}", ts, rand_counter())
}

/// Simple atomic counter for uniqueness within the same millisecond.
fn rand_counter() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Get the current timestamp in ISO 8601 format.
pub fn now_iso8601() -> String {
    let now = chrono::Utc::now();
    now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// Create a new ActivityLog entry with auto-generated id and timestamp.
pub fn create_entry(
    activity_type: ActivityType,
    agent_id: Option<String>,
    summary: String,
    details: serde_json::Value,
) -> ActivityLog {
    ActivityLog {
        id: generate_activity_id(),
        timestamp: now_iso8601(),
        activity_type,
        agent_id,
        workspace_id: None,
        summary,
        details,
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let store = ActivityStore::new(dir.path());

        let entry1 = ActivityLog {
            id: "act-1".to_string(),
            timestamp: "2026-04-10T10:00:00Z".to_string(),
            activity_type: ActivityType::AgentCreated,
            agent_id: Some("agent-1".to_string()),
            workspace_id: None,
            summary: "Agent Foo created".to_string(),
            details: serde_json::json!({}),
        };

        let entry2 = ActivityLog {
            id: "act-2".to_string(),
            timestamp: "2026-04-10T10:01:00Z".to_string(),
            activity_type: ActivityType::ConversationStarted,
            agent_id: Some("agent-1".to_string()),
            workspace_id: None,
            summary: "Conversation started".to_string(),
            details: serde_json::json!({}),
        };

        store.append(&entry1).unwrap();
        store.append(&entry2).unwrap();

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "act-1");
        assert_eq!(loaded[1].id, "act-2");
    }

    #[test]
    fn test_load_filtered() {
        let dir = tempfile::tempdir().unwrap();
        let store = ActivityStore::new(dir.path());

        for i in 0..5 {
            let entry = ActivityLog {
                id: format!("act-{}", i),
                timestamp: format!("2026-04-10T10:0{}:00Z", i),
                activity_type: ActivityType::AgentCreated,
                agent_id: if i % 2 == 0 {
                    Some("agent-a".to_string())
                } else {
                    Some("agent-b".to_string())
                },
                workspace_id: None,
                summary: format!("Activity {}", i),
                details: serde_json::json!({}),
            };
            store.append(&entry).unwrap();
        }

        // Filter by agent-a (should get 3: indices 0, 2, 4)
        let filtered = store.load_filtered(Some("agent-a"), 0, 10).unwrap();
        assert_eq!(filtered.len(), 3);

        // Newest first
        assert_eq!(filtered[0].id, "act-4");
        assert_eq!(filtered[1].id, "act-2");
        assert_eq!(filtered[2].id, "act-0");

        // Pagination
        let page = store.load_filtered(Some("agent-a"), 1, 2).unwrap();
        assert_eq!(page.len(), 2);
        assert_eq!(page[0].id, "act-2");
        assert_eq!(page[1].id, "act-0");
    }

    #[test]
    fn test_clear() {
        let dir = tempfile::tempdir().unwrap();
        let store = ActivityStore::new(dir.path());

        let entry = ActivityLog {
            id: "act-1".to_string(),
            timestamp: "2026-04-10T10:00:00Z".to_string(),
            activity_type: ActivityType::System,
            agent_id: None,
            workspace_id: None,
            summary: "Test".to_string(),
            details: serde_json::json!({}),
        };

        store.append(&entry).unwrap();
        assert!(store.file_path.exists());

        store.clear().unwrap();
        assert!(!store.file_path.exists());

        // Clear on non-existent file is ok
        store.clear().unwrap();
    }

    #[test]
    fn test_load_empty() {
        let dir = tempfile::tempdir().unwrap();
        let store = ActivityStore::new(dir.path());

        let loaded = store.load_all().unwrap();
        assert!(loaded.is_empty());
    }
}
