//! SQLite database initialization, connection management, and migrations.
//!
//! Provides a singleton-style database connection for the application.
//! The database file is stored in the workspace root as `agentszone.db`.
//!
//! ## Migration Strategy
//!
//! Migrations are embedded SQL strings executed in order. Each migration
//! is wrapped in a transaction for atomicity. A `schema_version` metadata
//! table tracks which migrations have been applied.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, Transaction};

// ===========================================================================
// Error type
// ===========================================================================

/// Database operation errors.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("migration error: {0}")]
    Migration(String),
}

// ===========================================================================
// Database connection management
// ===========================================================================

/// Database file name within the workspace root.
const DB_FILENAME: &str = "agentszone.db";

/// Get the database file path for a given workspace root.
pub fn db_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(DB_FILENAME)
}

/// Open a connection to the SQLite database at the given path.
///
/// Creates the file if it does not exist. Enables WAL mode for
/// better concurrent read performance.
fn open_connection(db_path: &Path) -> Result<Connection, DbError> {
    let conn = Connection::open(db_path)?;

    // Enable WAL mode for better concurrent reads
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    Ok(conn)
}

/// Initialize the database: create tables if they don't exist and run migrations.
///
/// This should be called once during application startup.
pub fn init_database(workspace_root: &Path) -> Result<Connection, DbError> {
    let path = db_path(workspace_root);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| DbError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    let mut conn = open_connection(&path)?;

    // Create schema_version table
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    )?;

    // Get current version
    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Run migrations
    run_migrations(&mut conn, current_version)?;

    log::info!(
        "[DB] Database initialized at {} (version: {})",
        path.display(),
        current_version
    );

    Ok(conn)
}

// ===========================================================================
// Migrations
// ===========================================================================

/// A single migration definition.
struct Migration {
    version: i64,
    description: &'static str,
    sql: &'static str,
}

/// All migrations in order.
fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "initial tables",
            sql: include_str!("migrations/V001__initial.sql"),
        },
        Migration {
            version: 2,
            description: "channel messages jsonl path",
            sql: include_str!("migrations/V002__channel_messages_jsonl.sql"),
        },
        Migration {
            version: 3,
            description: "data import marker",
            sql: include_str!("migrations/V003__data_import.sql"),
        },
        Migration {
            version: 4,
            description: "tasks v2 full data model",
            sql: include_str!("migrations/V004__tasks_v2.sql"),
        },
    ]
}

/// Run all pending migrations within transactions.
fn run_migrations(conn: &mut Connection, current_version: i64) -> Result<(), DbError> {
    let migrations = migrations();

    for migration in migrations {
        if migration.version <= current_version {
            continue;
        }

        log::info!(
            "[DB] Running migration V{}: {}",
            migration.version,
            migration.description
        );

        let tx = conn.transaction()?;
        execute_migration(&tx, &migration)?;
        tx.commit()?;

        log::info!(
            "[DB] Migration V{} applied successfully",
            migration.version
        );
    }

    Ok(())
}

/// Execute a single migration.
fn execute_migration(tx: &Transaction, migration: &Migration) -> Result<(), DbError> {
    // Execute the SQL
    tx.execute_batch(migration.sql)
        .map_err(|e| DbError::Migration(format!("V{}: {}", migration.version, e)))?;

    // Record the migration version
    tx.execute(
        "INSERT INTO schema_version (version) VALUES (?1)",
        params![migration.version],
    )?;

    Ok(())
}

// ===========================================================================
// Data migration from existing JSON files
// ===========================================================================

/// Import existing JSON file data into SQLite tables.
///
/// This is called once after database initialization. It checks if the
/// tables are empty and imports data from existing JSON/JSONL files.
/// The function is idempotent -- it only imports if tables are empty.
///
/// Original JSON/JSONL files are preserved as backups after import.
pub fn migrate_from_files(conn: &Connection, workspace_root: &Path) -> Result<(), DbError> {
    use super::db_helpers;

    // Check if agents table is empty (indicates first-time import needed)
    let agent_count = db_helpers::table_row_count(conn, "agents")?;
    if agent_count > 0 {
        log::info!("[DB] Data already imported ({} agents), skipping migration", agent_count);
        return Ok(());
    }

    log::info!("[DB] Starting data migration from JSON files...");

    // Import agents from workspace directories
    let agents_dir = workspace_root.join("agents");
    if agents_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&agents_dir) {
            for entry in entries.flatten() {
                if !entry.path().is_dir() {
                    continue;
                }
                let agent_id = entry.file_name().to_string_lossy().to_string();

                // Try to read IDENTITY.md to get agent metadata
                let identity_path = entry.path().join("IDENTITY.md");
                let (name, emoji, description) = if identity_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&identity_path) {
                        parse_identity_md(&content, &agent_id)
                    } else {
                        (agent_id.clone(), "robot".to_string(), String::new())
                    }
                } else {
                    (agent_id.clone(), "robot".to_string(), String::new())
                };

                let now = now_iso();

                let agent_row = db_helpers::AgentRow {
                    id: agent_id.clone(),
                    name,
                    emoji,
                    avatar_path: None,
                    enabled: true,
                    runtime_type: "claude-code".to_string(),
                    description,
                    created_at: now.clone(),
                    updated_at: now,
                };
                if let Err(e) = db_helpers::insert_agent(conn, &agent_row) {
                    log::warn!("[DB] Failed to import agent {}: {}", agent_id, e);
                } else {
                    log::info!("[DB] Imported agent: {}", agent_id);
                }
            }
        }
    }

    // Import channels from channels directory
    let channels_dir = workspace_root.join("channels");
    if channels_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&channels_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with("channel_") {
                    continue;
                }

                match std::fs::read_to_string(&path) {
                    Ok(data) => {
                        // Parse channel JSON
                        if let Ok(channel_val) = serde_json::from_str::<serde_json::Value>(&data) {
                            let channel_id = channel_val.get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let channel_name = channel_val.get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unnamed")
                                .to_string();
                            let created_at = channel_val.get("created_at")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let updated_at = channel_val.get("updated_at")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            if !channel_id.is_empty() {
                                let channel_row = db_helpers::ChannelRow {
                                    id: channel_id.clone(),
                                    name: channel_name,
                                    messages_jsonl_path: None,
                                    created_at: if created_at.is_empty() { now_iso() } else { created_at },
                                    updated_at: if updated_at.is_empty() { now_iso() } else { updated_at },
                                };
                                if let Err(e) = db_helpers::insert_channel(conn, &channel_row) {
                                    log::warn!("[DB] Failed to import channel {}: {}", channel_id, e);
                                } else {
                                    log::info!("[DB] Imported channel: {}", channel_id);

                                    // Import channel members
                                    if let Some(members) = channel_val.get("members").and_then(|v| v.as_array()) {
                                        for member in members {
                                            let member_row = db_helpers::ChannelMemberRow {
                                                channel_id: channel_id.clone(),
                                                agent_id: member.get("agent_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                                role: member.get("role").and_then(|v| v.as_str()).unwrap_or("member").to_string(),
                                                joined_at: member.get("joined_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                            };
                                            if !member_row.agent_id.is_empty() {
                                                if let Err(e) = db_helpers::insert_channel_member(conn, &member_row) {
                                                    log::warn!("[DB] Failed to import channel member: {}", e);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("[DB] Failed to read channel file {:?}: {}", path, e);
                    }
                }
            }
        }
    }

    // Import threads from agent conversation directories
    if agents_dir.is_dir() {
        if let Ok(agent_entries) = std::fs::read_dir(&agents_dir) {
            for agent_entry in agent_entries.flatten() {
                if !agent_entry.path().is_dir() {
                    continue;
                }
                let agent_id = agent_entry.file_name().to_string_lossy().to_string();
                let conv_dir = agent_entry.path().join("conversations");

                if !conv_dir.is_dir() {
                    continue;
                }

                if let Ok(conv_entries) = std::fs::read_dir(&conv_dir) {
                    for conv_entry in conv_entries.flatten() {
                        let path = conv_entry.path();
                        if path.extension().and_then(|e| e.to_str()) != Some("json") {
                            continue;
                        }
                        let fname = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                        if !fname.starts_with("thread_") {
                            continue;
                        }

                        match std::fs::read_to_string(&path) {
                            Ok(data) => {
                                if let Ok(thread_val) = serde_json::from_str::<serde_json::Value>(&data) {
                                    let thread_id = thread_val.get("id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or(fname.strip_prefix("thread_").unwrap_or(""))
                                        .to_string();
                                    let title = thread_val.get("title")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let session_id = thread_val.get("session_id")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string());
                                    let message_count = thread_val.get("messages")
                                        .and_then(|v| v.as_array())
                                        .map(|a| a.len() as i64)
                                        .unwrap_or(0);
                                    let jsonl_path = format!("agents/{}/conversations/threads/{}.jsonl", agent_id, thread_id);
                                    let created_at = thread_val.get("created_at")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let updated_at = thread_val.get("updated_at")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();

                                    let thread_row = db_helpers::ThreadRow {
                                        id: thread_id.clone(),
                                        agent_id: agent_id.clone(),
                                        title,
                                        session_id,
                                        message_count,
                                        jsonl_path: Some(jsonl_path),
                                        created_at: if created_at.is_empty() { now_iso() } else { created_at },
                                        updated_at: if updated_at.is_empty() { now_iso() } else { updated_at },
                                    };
                                    if let Err(e) = db_helpers::insert_thread(conn, &thread_row) {
                                        log::warn!("[DB] Failed to import thread {}: {}", thread_id, e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("[DB] Failed to read thread file {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }
    }

    // Import activity log from JSONL file
    let activity_jsonl = workspace_root.join("activity.jsonl");
    if activity_jsonl.exists() {
        use std::io::BufRead;
        if let Ok(file) = std::fs::File::open(&activity_jsonl) {
            let reader = std::io::BufReader::new(file);
            let mut imported = 0u32;
            for line in reader.lines() {
                if let Ok(line) = line {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        let id = val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let timestamp = val.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let activity_type = val.get("activity_type").and_then(|v| v.as_str()).unwrap_or("system").to_string();
                        let agent_id = val.get("agent_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let workspace_id = val.get("workspace_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let summary = val.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let details_json = val.get("details")
                            .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string()))
                            .unwrap_or_else(|| "{}".to_string());

                        if !id.is_empty() {
                            let row = db_helpers::ActivityLogRow {
                                id,
                                timestamp: if timestamp.is_empty() { now_iso() } else { timestamp },
                                activity_type,
                                agent_id,
                                workspace_id,
                                summary,
                                details_json,
                            };
                            if db_helpers::insert_activity(conn, &row).is_ok() {
                                imported += 1;
                            }
                        }
                    }
                }
            }
            log::info!("[DB] Imported {} activity log entries", imported);
        }
    }

    // Import skills from agent skill directories
    if agents_dir.is_dir() {
        if let Ok(agent_entries) = std::fs::read_dir(&agents_dir) {
            for agent_entry in agent_entries.flatten() {
                if !agent_entry.path().is_dir() {
                    continue;
                }
                let agent_id = agent_entry.file_name().to_string_lossy().to_string();
                let skills_file = agent_entry.path().join("skills").join("skills.json");

                if !skills_file.exists() {
                    continue;
                }

                match std::fs::read_to_string(&skills_file) {
                    Ok(data) => {
                        if let Ok(skills_val) = serde_json::from_str::<serde_json::Value>(&data) {
                            if let Some(skills) = skills_val.as_array() {
                                for skill in skills {
                                    let id = skill.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let name = skill.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let skill_type = skill.get("skill_type").and_then(|v| v.as_str()).unwrap_or("tool").to_string();
                                    let status = skill.get("status").and_then(|v| v.as_str()).unwrap_or("active").to_string();
                                    let config_json = skill.get("config")
                                        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string()))
                                        .unwrap_or_else(|| "{}".to_string());
                                    let created_at = skill.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let updated_at = skill.get("updated_at").and_then(|v| v.as_str()).unwrap_or("").to_string();

                                    if !id.is_empty() {
                                        let row = db_helpers::SkillRow {
                                            id,
                                            agent_id: agent_id.clone(),
                                            name,
                                            skill_type,
                                            status,
                                            config_json,
                                            created_at: if created_at.is_empty() { now_iso() } else { created_at },
                                            updated_at: if updated_at.is_empty() { now_iso() } else { updated_at },
                                        };
                                        if let Err(e) = db_helpers::insert_skill(conn, &row) {
                                            log::warn!("[DB] Failed to import skill: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("[DB] Failed to read skills file {:?}: {}", skills_file, e);
                    }
                }
            }
        }
    }

    log::info!("[DB] Data migration from JSON files completed");
    Ok(())
}

/// Parse IDENTITY.md to extract name, emoji, and description.
fn parse_identity_md(content: &str, default_name: &str) -> (String, String, String) {
    let mut name = default_name.to_string();
    let mut emoji = "robot".to_string();
    let mut description = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("Name:") {
            name = line.strip_prefix("Name:").unwrap_or(default_name).trim().to_string();
        } else if line.starts_with("Emoji:") {
            emoji = line.strip_prefix("Emoji:").unwrap_or("robot").trim().to_string();
        } else if line.starts_with("Description:") {
            description = line.strip_prefix("Description:").unwrap_or("").trim().to_string();
        }
    }

    (name, emoji, description)
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
    let (year, month, day) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hours, minutes, seconds)
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let diy = if is_leap(year) { 366 } else { 365 };
        if days < diy { break; }
        days -= diy;
        year += 1;
    }
    let leap = is_leap(year);
    let md: [u64; 12] = if leap { [31,29,31,30,31,30,31,31,30,31,30,31] } else { [31,28,31,30,31,30,31,31,30,31,30,31] };
    let mut month = 1u64;
    for &x in &md { if days < x { break; } days -= x; month += 1; }
    (year, month, days + 1)
}

fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_database_creates_tables() {
        let dir = tempfile::tempdir().unwrap();
        let conn = init_database(dir.path()).unwrap();

        // Verify tables exist
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"agents".to_string()));
        assert!(tables.contains(&"channels".to_string()));
        assert!(tables.contains(&"channel_members".to_string()));
        assert!(tables.contains(&"threads".to_string()));
        assert!(tables.contains(&"tasks".to_string()));
        assert!(tables.contains(&"skills".to_string()));
        assert!(tables.contains(&"activity_log".to_string()));
        assert!(tables.contains(&"schema_version".to_string()));
    }

    #[test]
    fn test_init_database_idempotent() {
        let dir = tempfile::tempdir().unwrap();

        // Init twice should succeed
        let conn1 = init_database(dir.path()).unwrap();
        drop(conn1);
        let conn2 = init_database(dir.path()).unwrap();

        // Schema version should be the latest migration
        let version: i64 = conn2
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(version >= 1);
    }

    #[test]
    fn test_schema_version_tracks_migrations() {
        let dir = tempfile::tempdir().unwrap();
        let conn = init_database(dir.path()).unwrap();

        let versions: Vec<i64> = conn
            .prepare("SELECT version FROM schema_version ORDER BY version")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        // Should have at least version 1
        assert!(!versions.is_empty());
        assert_eq!(versions[0], 1);
    }
}
