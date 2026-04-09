//! Thread data model for 1-on-1 Agent conversations.
//!
//! A Thread represents a single conversation between a user and an Agent.
//! Threads are stored as JSON files in the agent's `conversations/` directory.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ===========================================================================
// Thread data model
// ===========================================================================

/// A conversation thread between user and a single Agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    /// Unique thread identifier.
    pub id: String,
    /// The agent this thread belongs to.
    pub agent_id: String,
    /// Display title for the thread.
    pub title: String,
    /// Claude Code session ID for resuming conversations.
    pub session_id: Option<String>,
    /// Messages in this thread.
    pub messages: Vec<ThreadMessage>,
    /// Creation timestamp (ISO 8601).
    pub created_at: String,
    /// Last update timestamp (ISO 8601).
    pub updated_at: String,
}

/// A single message within a Thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMessage {
    /// Unique message identifier.
    pub id: String,
    /// Role: "user" or "agent".
    pub role: String,
    /// Message text content.
    pub content: String,
    /// Timestamp (ISO 8601).
    pub timestamp: String,
}

/// Lightweight thread info for listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    pub id: String,
    pub agent_id: String,
    pub title: String,
    /// Preview: first 80 chars of the last message.
    pub preview: String,
    pub message_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

// ===========================================================================
// Thread storage
// ===========================================================================

/// Manages Thread persistence on disk within an agent's conversations/ directory.
pub struct ThreadStore<'a> {
    conversations_dir: &'a Path,
}

impl<'a> ThreadStore<'a> {
    /// Create a new ThreadStore for the given conversations directory.
    pub fn new(conversations_dir: &'a Path) -> Self {
        Self { conversations_dir }
    }

    /// Ensure the conversations directory exists.
    fn ensure_dir(&self) -> Result<(), ThreadError> {
        fs::create_dir_all(self.conversations_dir).map_err(|e| ThreadError::Io {
            path: self.conversations_dir.to_path_buf(),
            source: e,
        })
    }

    /// Get the file path for a thread by ID.
    fn thread_file(&self, thread_id: &str) -> PathBuf {
        self.conversations_dir.join(format!("thread_{}.json", thread_id))
    }

    /// Save a thread to disk.
    pub fn save(&self, thread: &Thread) -> Result<(), ThreadError> {
        self.ensure_dir()?;
        let path = self.thread_file(&thread.id);
        let json = serde_json::to_string_pretty(thread).map_err(|e| ThreadError::Serialization {
            message: e.to_string(),
        })?;
        fs::write(&path, json).map_err(|e| ThreadError::Io {
            path,
            source: e,
        })
    }

    /// Load a thread from disk by ID.
    pub fn load(&self, thread_id: &str) -> Result<Thread, ThreadError> {
        let path = self.thread_file(thread_id);
        let data = fs::read_to_string(&path).map_err(|e| ThreadError::Io {
            path: path.clone(),
            source: e,
        })?;
        serde_json::from_str(&data).map_err(|e| ThreadError::Serialization {
            message: e.to_string(),
        })
    }

    /// Delete a thread from disk by ID.
    pub fn delete(&self, thread_id: &str) -> Result<(), ThreadError> {
        let path = self.thread_file(thread_id);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| ThreadError::Io {
                path,
                source: e,
            })
        } else {
            Err(ThreadError::NotFound {
                thread_id: thread_id.to_string(),
            })
        }
    }

    /// List all threads (as lightweight ThreadInfo).
    pub fn list(&self) -> Result<Vec<ThreadInfo>, ThreadError> {
        self.ensure_dir()?;
        let mut threads = Vec::new();

        let entries = fs::read_dir(self.conversations_dir).map_err(|e| ThreadError::Io {
            path: self.conversations_dir.to_path_buf(),
            source: e,
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| ThreadError::Io {
                path: self.conversations_dir.to_path_buf(),
                source: e,
            })?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with("thread_") {
                    continue;
                }

                match fs::read_to_string(&path) {
                    Ok(data) => match serde_json::from_str::<Thread>(&data) {
                        Ok(thread) => {
                            let preview = thread
                                .messages
                                .last()
                                .map(|m| {
                                    let content = &m.content;
                                    if content.len() > 80 {
                                        format!("{}...", &content[..80])
                                    } else {
                                        content.clone()
                                    }
                                })
                                .unwrap_or_default();

                            threads.push(ThreadInfo {
                                id: thread.id,
                                agent_id: thread.agent_id,
                                title: thread.title,
                                preview,
                                message_count: thread.messages.len(),
                                created_at: thread.created_at,
                                updated_at: thread.updated_at,
                            });
                        }
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        threads.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(threads)
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Generate a simple unique ID (timestamp-based with random suffix).
pub fn generate_id() -> String {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    // Use lower 16 hex chars for a compact ID
    format!("{:016x}", nanos & 0xFFFF_FFFF_FFFF_FFFF)
}

/// Get current ISO 8601 timestamp.
pub fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple UTC ISO format
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days: [u64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

// ===========================================================================
// Error type
// ===========================================================================

#[derive(Debug, thiserror::Error)]
pub enum ThreadError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("thread not found: {thread_id}")]
    NotFound { thread_id: String },

    #[error("serialization error: {message}")]
    Serialization { message: String },
}
