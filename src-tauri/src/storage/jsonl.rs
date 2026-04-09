//! JSONL (JSON Lines) storage engine for conversation persistence.
//!
//! Each conversation thread is stored as a JSONL file where each line is a
//! JSON object representing a single message. This append-only format provides:
//!
//! - Efficient message appending (no need to rewrite the entire file)
//! - Crash safety (partial writes only affect the last line)
//! - Simple format for streaming reads
//!
//! File path pattern:
//!   `agents/{agent_id}/conversations/threads/{thread_id}.jsonl`
//!
//! Each JSONL line follows this format:
//!   `{"role":"user"|"agent","content":"...","timestamp":"...","id":"...","thread_id":"..."}`

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::workspace::thread::ThreadMessage;

// ===========================================================================
// JSONL Message Record
// ===========================================================================

/// A single message record stored in JSONL format.
///
/// This is the on-disk representation of a message, including the `thread_id`
/// for cross-referencing with the thread metadata JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonlMessage {
    /// Unique message identifier.
    pub id: String,
    /// Role: "user", "agent", or "system".
    pub role: String,
    /// Message text content.
    pub content: String,
    /// Timestamp (ISO 8601).
    pub timestamp: String,
    /// Thread ID this message belongs to.
    pub thread_id: String,
}

impl From<&ThreadMessage> for JsonlMessage {
    fn from(msg: &ThreadMessage) -> Self {
        Self {
            id: msg.id.clone(),
            role: msg.role.clone(),
            content: msg.content.clone(),
            timestamp: msg.timestamp.clone(),
            thread_id: String::new(), // caller must set this
        }
    }
}

impl From<JsonlMessage> for ThreadMessage {
    fn from(msg: JsonlMessage) -> Self {
        Self {
            id: msg.id,
            role: msg.role,
            content: msg.content,
            timestamp: msg.timestamp,
        }
    }
}

// ===========================================================================
// JSONL Store
// ===========================================================================

/// Append-only JSONL storage for conversation messages.
///
/// Each thread gets its own JSONL file. Messages are appended one line at a
/// time, making writes efficient and crash-safe.
pub struct JsonlStore {
    /// Base directory for thread JSONL files.
    /// Typically: `{workspace_root}/agents/{agent_id}/conversations/threads/`
    threads_dir: PathBuf,
}

impl JsonlStore {
    /// Create a new JsonlStore for the given threads directory.
    pub fn new(threads_dir: impl Into<PathBuf>) -> Self {
        Self {
            threads_dir: threads_dir.into(),
        }
    }

    /// Ensure the threads directory exists.
    fn ensure_dir(&self) -> Result<(), JsonlError> {
        fs::create_dir_all(&self.threads_dir).map_err(|e| JsonlError::Io {
            path: self.threads_dir.clone(),
            source: e,
        })
    }

    /// Get the JSONL file path for a thread.
    fn thread_file(&self, thread_id: &str) -> PathBuf {
        self.threads_dir.join(format!("{}.jsonl", thread_id))
    }

    /// Append a single message to a thread's JSONL file.
    ///
    /// Creates the file (and directory) if it does not exist.
    /// Each message is written as a single JSON line followed by a newline.
    pub fn append_message(
        &self,
        thread_id: &str,
        message: &ThreadMessage,
    ) -> Result<(), JsonlError> {
        self.ensure_dir()?;

        let path = self.thread_file(thread_id);
        let record = JsonlMessage {
            id: message.id.clone(),
            role: message.role.clone(),
            content: message.content.clone(),
            timestamp: message.timestamp.clone(),
            thread_id: thread_id.to_string(),
        };

        let mut line = serde_json::to_string(&record).map_err(|e| JsonlError::Serialization {
            message: e.to_string(),
        })?;
        line.push('\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| JsonlError::Io {
                path: path.clone(),
                source: e,
            })?;

        file.write_all(line.as_bytes()).map_err(|e| JsonlError::Io {
            path: path.clone(),
            source: e,
        })?;

        file.flush().map_err(|e| JsonlError::Io {
            path,
            source: e,
        })?;

        Ok(())
    }

    /// Load all messages from a thread's JSONL file.
    ///
    /// Returns messages in chronological order (oldest first).
    /// Malformed lines are skipped with a warning log.
    pub fn load_messages(&self, thread_id: &str) -> Result<Vec<ThreadMessage>, JsonlError> {
        let path = self.thread_file(thread_id);

        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&path).map_err(|e| JsonlError::Io {
            path: path.clone(),
            source: e,
        })?;

        let reader = BufReader::new(file);
        let mut messages = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = match line_result {
                Ok(l) => l,
                Err(e) => {
                    log::warn!(
                        "[JsonlStore] Failed to read line {} in {}: {}",
                        line_num + 1,
                        path.display(),
                        e
                    );
                    continue;
                }
            };

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            match serde_json::from_str::<JsonlMessage>(trimmed) {
                Ok(record) => messages.push(ThreadMessage::from(record)),
                Err(e) => {
                    log::warn!(
                        "[JsonlStore] Failed to parse line {} in {}: {}",
                        line_num + 1,
                        path.display(),
                        e
                    );
                    continue;
                }
            }
        }

        Ok(messages)
    }

    /// Load the most recent N messages from a thread's JSONL file.
    ///
    /// Useful for previewing conversations without loading the full history.
    /// Returns messages in chronological order.
    pub fn load_recent_messages(
        &self,
        thread_id: &str,
        limit: usize,
    ) -> Result<Vec<ThreadMessage>, JsonlError> {
        let mut all = self.load_messages(thread_id)?;
        if all.len() <= limit {
            return Ok(all);
        }
        Ok(all.split_off(all.len() - limit))
    }

    /// Delete a thread's JSONL file.
    pub fn delete_thread(&self, thread_id: &str) -> Result<(), JsonlError> {
        let path = self.thread_file(thread_id);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| JsonlError::Io {
                path,
                source: e,
            })?;
        }
        Ok(())
    }

    /// List all thread IDs that have JSONL files.
    pub fn list_thread_ids(&self) -> Result<Vec<String>, JsonlError> {
        if !self.threads_dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut ids = Vec::new();
        let entries = fs::read_dir(&self.threads_dir).map_err(|e| JsonlError::Io {
            path: self.threads_dir.clone(),
            source: e,
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| JsonlError::Io {
                path: self.threads_dir.clone(),
                source: e,
            })?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    ids.push(stem.to_string());
                }
            }
        }

        ids.sort();
        Ok(ids)
    }
}

// ===========================================================================
// Error type
// ===========================================================================

/// JSONL storage errors.
#[derive(Debug, thiserror::Error)]
pub enum JsonlError {
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
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as IoWrite;

    fn make_msg(id: &str, role: &str, content: &str) -> ThreadMessage {
        ThreadMessage {
            id: id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            timestamp: "2026-04-09T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_append_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let store = JsonlStore::new(dir.path().join("threads"));

        let msg1 = make_msg("m1", "user", "Hello");
        let msg2 = make_msg("m2", "agent", "Hi there!");

        store.append_message("t1", &msg1).unwrap();
        store.append_message("t1", &msg2).unwrap();

        let loaded = store.load_messages("t1").unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "m1");
        assert_eq!(loaded[0].role, "user");
        assert_eq!(loaded[1].id, "m2");
        assert_eq!(loaded[1].role, "agent");
    }

    #[test]
    fn test_load_nonexistent_thread() {
        let dir = tempfile::tempdir().unwrap();
        let store = JsonlStore::new(dir.path().join("threads"));

        let loaded = store.load_messages("nonexistent").unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_load_recent_messages() {
        let dir = tempfile::tempdir().unwrap();
        let store = JsonlStore::new(dir.path().join("threads"));

        for i in 0..10 {
            let msg = make_msg(&format!("m{}", i), "user", &format!("msg {}", i));
            store.append_message("t1", &msg).unwrap();
        }

        // Load only the last 3 messages
        let recent = store.load_recent_messages("t1", 3).unwrap();
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].id, "m7");
        assert_eq!(recent[1].id, "m8");
        assert_eq!(recent[2].id, "m9");
    }

    #[test]
    fn test_delete_thread() {
        let dir = tempfile::tempdir().unwrap();
        let store = JsonlStore::new(dir.path().join("threads"));

        let msg = make_msg("m1", "user", "Hello");
        store.append_message("t1", &msg).unwrap();
        assert!(store.thread_file("t1").exists());

        store.delete_thread("t1").unwrap();
        assert!(!store.thread_file("t1").exists());
    }

    #[test]
    fn test_list_thread_ids() {
        let dir = tempfile::tempdir().unwrap();
        let store = JsonlStore::new(dir.path().join("threads"));

        let msg = make_msg("m1", "user", "Hello");
        store.append_message("thread-aaa", &msg).unwrap();
        store.append_message("thread-bbb", &msg).unwrap();
        store.append_message("thread-ccc", &msg).unwrap();

        let ids = store.list_thread_ids().unwrap();
        assert_eq!(ids, vec!["thread-aaa", "thread-bbb", "thread-ccc"]);
    }

    #[test]
    fn test_malformed_lines_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let store = JsonlStore::new(dir.path().join("threads"));

        // Write a file with some malformed lines manually
        store.ensure_dir().unwrap();
        let path = store.thread_file("t1");
        let mut file = File::create(&path).unwrap();
        writeln!(file, "{{\"id\":\"m1\",\"role\":\"user\",\"content\":\"hello\",\"timestamp\":\"2026-04-09T12:00:00Z\",\"thread_id\":\"t1\"}}").unwrap();
        writeln!(file, "this is not valid json").unwrap();
        writeln!(file, "").unwrap(); // empty line
        writeln!(file, "{{\"id\":\"m2\",\"role\":\"agent\",\"content\":\"hi\",\"timestamp\":\"2026-04-09T12:00:01Z\",\"thread_id\":\"t1\"}}").unwrap();

        let loaded = store.load_messages("t1").unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "m1");
        assert_eq!(loaded[1].id, "m2");
    }

    #[test]
    fn test_append_creates_directory() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("deep").join("nested").join("threads");
        let store = JsonlStore::new(&nested);

        assert!(!nested.exists());
        let msg = make_msg("m1", "user", "Hello");
        store.append_message("t1", &msg).unwrap();
        assert!(nested.exists());
    }
}
