//! Agent Workspace management.
//!
//! Each Agent gets an isolated directory under `workspaces/agents/<name>/`
//! containing conversations, context snapshots, output, skills, and config.

use std::fs;
use std::path::{Path, PathBuf};

/// Subdirectory names within an Agent workspace.
pub const DIR_CONVERSATIONS: &str = "conversations";
pub const DIR_CONTEXT: &str = "context";
pub const DIR_OUTPUT: &str = "output";
pub const DIR_SKILLS: &str = "skills";
pub const DIR_CONFIG: &str = "config";

/// All Agent workspace subdirectories.
const AGENT_SUBDIRS: &[&str] = &[
    DIR_CONVERSATIONS,
    DIR_CONTEXT,
    DIR_OUTPUT,
    DIR_SKILLS,
    DIR_CONFIG,
];

/// File naming convention for conversations (JSONL).
/// Pattern: `<timestamp>_<session_id>.jsonl`
pub const CONVERSATION_FILE_PATTERN: &str = r"^\d{8}T\d{6}_[a-f0-9]{8}\.jsonl$";

/// File naming convention for context snapshots.
/// Pattern: `<timestamp>_snapshot.md`
pub const CONTEXT_FILE_PATTERN: &str = r"^\d{8}T\d{6}_snapshot\.md$";

/// An isolated Agent workspace on disk.
///
/// Manages the physical directory structure for a single Agent,
/// ensuring that conversation logs, context, outputs, and config
/// are kept separate from other Agents.
#[derive(Debug, Clone)]
pub struct AgentWorkspace {
    /// Root path of this Agent's workspace.
    base_path: PathBuf,
}

impl AgentWorkspace {
    /// Create a new AgentWorkspace handle for the given base path.
    ///
    /// Does NOT create directories on disk; call [`Self::initialize`]
    /// to materialize the directory structure.
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// The root path of this workspace.
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Path to the `conversations/` directory.
    pub fn conversations_dir(&self) -> PathBuf {
        self.base_path.join(DIR_CONVERSATIONS)
    }

    /// Path to the `context/` directory.
    pub fn context_dir(&self) -> PathBuf {
        self.base_path.join(DIR_CONTEXT)
    }

    /// Path to the `output/` directory.
    pub fn output_dir(&self) -> PathBuf {
        self.base_path.join(DIR_OUTPUT)
    }

    /// Path to the `skills/` directory.
    pub fn skills_dir(&self) -> PathBuf {
        self.base_path.join(DIR_SKILLS)
    }

    /// Path to the `config/` directory.
    pub fn config_dir(&self) -> PathBuf {
        self.base_path.join(DIR_CONFIG)
    }

    /// Path to the Agent's `IDENTITY.md` file.
    pub fn identity_file(&self) -> PathBuf {
        self.base_path.join("IDENTITY.md")
    }

    /// Path to the Agent's `SOUL.md` file (overrides global).
    pub fn soul_file(&self) -> PathBuf {
        self.base_path.join("SOUL.md")
    }

    /// Check whether this workspace exists on disk.
    pub fn exists(&self) -> bool {
        self.base_path.is_dir()
    }

    /// Create the workspace directory structure on disk.
    ///
    /// Creates the base path and all standard subdirectories.
    /// Idempotent -- does nothing (returns Ok) if directories already exist.
    pub fn initialize(&self) -> Result<(), WorkspaceError> {
        fs::create_dir_all(&self.base_path).map_err(|e| WorkspaceError::Io {
            path: self.base_path.clone(),
            source: e,
        })?;

        for subdir in AGENT_SUBDIRS {
            let path = self.base_path.join(subdir);
            fs::create_dir_all(&path).map_err(|e| WorkspaceError::Io {
                path: path.clone(),
                source: e,
            })?;
        }

        Ok(())
    }

    /// Generate a conversation file path following the naming convention.
    ///
    /// Pattern: `conversations/<YYYYMMDD>T<HHmmSS>_<session_id>.jsonl`
    pub fn conversation_file(&self, session_id: &str) -> PathBuf {
        let timestamp = chrono_now_compact();
        self.conversations_dir()
            .join(format!("{timestamp}_{session_id}.jsonl"))
    }

    /// Generate a context snapshot file path.
    ///
    /// Pattern: `context/<YYYYMMDD>T<HHmmSS>_snapshot.md`
    pub fn context_snapshot_file(&self) -> PathBuf {
        let timestamp = chrono_now_compact();
        self.context_dir().join(format!("{timestamp}_snapshot.md"))
    }

    /// List all conversation files in this workspace, sorted by name (oldest first).
    pub fn list_conversations(&self) -> Result<Vec<PathBuf>, WorkspaceError> {
        list_files_matching(&self.conversations_dir(), |name| {
            name.ends_with(".jsonl")
        })
    }

    /// List all context snapshots, sorted by name (oldest first).
    pub fn list_context_snapshots(&self) -> Result<Vec<PathBuf>, WorkspaceError> {
        list_files_matching(&self.context_dir(), |name| {
            name.ends_with("_snapshot.md")
        })
    }

    /// Calculate disk usage of this workspace in bytes.
    pub fn disk_usage(&self) -> Result<u64, WorkspaceError> {
        if !self.exists() {
            return Ok(0);
        }
        Ok(dir_size(&self.base_path))
    }

    /// Delete the entire workspace directory.
    ///
    /// Use with caution -- this is irreversible.
    pub fn delete(&self) -> Result<(), WorkspaceError> {
        if self.base_path.exists() {
            fs::remove_dir_all(&self.base_path).map_err(|e| WorkspaceError::Io {
                path: self.base_path.clone(),
                source: e,
            })?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compact timestamp for file naming: `YYYYMMDDTHHmmSS`.
fn chrono_now_compact() -> String {
    // Use a simple hand-rolled formatter to avoid pulling in chrono.
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    // Rough conversion -- sufficient for file naming uniqueness.
    let secs = dur.as_secs();
    format_timestamp_compact(secs)
}

/// Convert unix timestamp to `YYYYMMDDTHHmmSS` (UTC).
fn format_timestamp_compact(secs: u64) -> String {
    // Simplified UTC date computation (no leap-second handling).
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Compute year/month/day from days since 1970-01-01.
    let (year, month, day) = days_to_ymd(days);

    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}",
        year, month, day, hours, minutes, seconds,
    )
}

/// Convert days since Unix epoch to (year, month, day) in UTC.
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

/// List files in a directory that match a predicate, sorted by name.
fn list_files_matching(
    dir: &Path,
    predicate: impl Fn(&str) -> bool,
) -> Result<Vec<PathBuf>, WorkspaceError> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| WorkspaceError::Io {
            path: dir.to_path_buf(),
            source: e,
        })?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            if predicate(&name) {
                Some(entry.path())
            } else {
                None
            }
        })
        .collect();
    files.sort();
    Ok(files)
}

/// Recursively calculate directory size in bytes.
fn dir_size(path: &Path) -> u64 {
    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| {
                    let m = fs::symlink_metadata(e.path());
                    match m {
                        Ok(m) if m.is_dir() => dir_size(&e.path()),
                        Ok(m) => m.len(),
                        Err(_) => 0,
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Workspace operation errors.
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("workspace not found: {0}")]
    NotFound(PathBuf),

    #[error("invalid workspace state: {0}")]
    InvalidState(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp_compact() {
        // 1970-01-01T00:00:00 UTC
        assert_eq!(format_timestamp_compact(0), "19700101T000000");
        // 2000-01-01T00:00:00 UTC (known value)
        let ts = format_timestamp_compact(946_684_800);
        assert_eq!(ts, "20000101T000000");
    }

    #[test]
    fn test_agent_workspace_paths() {
        let ws = AgentWorkspace::new("/tmp/test-workspace");
        assert_eq!(ws.conversations_dir(), PathBuf::from("/tmp/test-workspace/conversations"));
        assert_eq!(ws.context_dir(), PathBuf::from("/tmp/test-workspace/context"));
        assert_eq!(ws.output_dir(), PathBuf::from("/tmp/test-workspace/output"));
        assert_eq!(ws.skills_dir(), PathBuf::from("/tmp/test-workspace/skills"));
        assert_eq!(ws.config_dir(), PathBuf::from("/tmp/test-workspace/config"));
        assert_eq!(ws.identity_file(), PathBuf::from("/tmp/test-workspace/IDENTITY.md"));
        assert_eq!(ws.soul_file(), PathBuf::from("/tmp/test-workspace/SOUL.md"));
    }

    #[test]
    fn test_conversation_file_naming() {
        let ws = AgentWorkspace::new("/tmp/test");
        let path = ws.conversation_file("abcd1234");
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(name.ends_with("_abcd1234.jsonl"));
        // Format: YYYYMMDDTHHmmSS_abcd1234.jsonl
        assert!(name.len() == 30);
    }
}
