//! Database helper functions for common query patterns.
//!
//! Provides typed query helpers for agents, channels, threads, tasks,
//! skills, and activity log operations.

use rusqlite::{params, Connection, Row};

use super::db::DbError;

// ===========================================================================
// Agent queries
// ===========================================================================

/// Agent row from the database.
#[derive(Debug, Clone)]
pub struct AgentRow {
    pub id: String,
    pub name: String,
    pub emoji: String,
    pub avatar_path: Option<String>,
    pub enabled: bool,
    pub runtime_type: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

impl AgentRow {
    fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            emoji: row.get("emoji")?,
            avatar_path: row.get("avatar_path")?,
            enabled: row.get::<_, i64>("enabled")? != 0,
            runtime_type: row.get("runtime_type")?,
            description: row.get("description")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// Insert an agent into the database.
pub fn insert_agent(conn: &Connection, agent: &AgentRow) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR REPLACE INTO agents (id, name, emoji, avatar_path, enabled, runtime_type, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            agent.id,
            agent.name,
            agent.emoji,
            agent.avatar_path,
            agent.enabled as i64,
            agent.runtime_type,
            agent.description,
            agent.created_at,
            agent.updated_at,
        ],
    )?;
    Ok(())
}

/// List all enabled agents from the database.
pub fn list_agents(conn: &Connection) -> Result<Vec<AgentRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM agents WHERE enabled = 1 ORDER BY name ASC"
    )?;
    let rows = stmt.query_map([], AgentRow::from_row)?;
    let mut agents = Vec::new();
    for row in rows {
        agents.push(row?);
    }
    Ok(agents)
}

/// Get a single agent by ID.
pub fn get_agent(conn: &Connection, agent_id: &str) -> Result<Option<AgentRow>, DbError> {
    let mut stmt = conn.prepare("SELECT * FROM agents WHERE id = ?1")?;
    let mut rows = stmt.query_map(params![agent_id], AgentRow::from_row)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Delete an agent from the database.
pub fn delete_agent(conn: &Connection, agent_id: &str) -> Result<(), DbError> {
    conn.execute("DELETE FROM agents WHERE id = ?1", params![agent_id])?;
    Ok(())
}

// ===========================================================================
// Channel queries
// ===========================================================================

/// Channel row from the database.
#[derive(Debug, Clone)]
pub struct ChannelRow {
    pub id: String,
    pub name: String,
    pub messages_jsonl_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl ChannelRow {
    fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            messages_jsonl_path: row.get("messages_jsonl_path")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// Channel member row.
#[derive(Debug, Clone)]
pub struct ChannelMemberRow {
    pub channel_id: String,
    pub agent_id: String,
    pub role: String,
    pub joined_at: String,
}

impl ChannelMemberRow {
    fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            channel_id: row.get("channel_id")?,
            agent_id: row.get("agent_id")?,
            role: row.get("role")?,
            joined_at: row.get("joined_at")?,
        })
    }
}

/// Insert a channel into the database.
pub fn insert_channel(conn: &Connection, channel: &ChannelRow) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR REPLACE INTO channels (id, name, messages_jsonl_path, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            channel.id,
            channel.name,
            channel.messages_jsonl_path,
            channel.created_at,
            channel.updated_at,
        ],
    )?;
    Ok(())
}

/// List all channels from the database, ordered by updated_at desc.
pub fn list_channels(conn: &Connection) -> Result<Vec<ChannelRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM channels ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map([], ChannelRow::from_row)?;
    let mut channels = Vec::new();
    for row in rows {
        channels.push(row?);
    }
    Ok(channels)
}

/// Get a single channel by ID.
pub fn get_channel(conn: &Connection, channel_id: &str) -> Result<Option<ChannelRow>, DbError> {
    let mut stmt = conn.prepare("SELECT * FROM channels WHERE id = ?1")?;
    let mut rows = stmt.query_map(params![channel_id], ChannelRow::from_row)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Delete a channel from the database.
pub fn delete_channel(conn: &Connection, channel_id: &str) -> Result<(), DbError> {
    conn.execute("DELETE FROM channels WHERE id = ?1", params![channel_id])?;
    Ok(())
}

/// Insert a channel member.
pub fn insert_channel_member(conn: &Connection, member: &ChannelMemberRow) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR REPLACE INTO channel_members (channel_id, agent_id, role, joined_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![member.channel_id, member.agent_id, member.role, member.joined_at],
    )?;
    Ok(())
}

/// Get all members of a channel.
pub fn get_channel_members(conn: &Connection, channel_id: &str) -> Result<Vec<ChannelMemberRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM channel_members WHERE channel_id = ?1"
    )?;
    let rows = stmt.query_map(params![channel_id], ChannelMemberRow::from_row)?;
    let mut members = Vec::new();
    for row in rows {
        members.push(row?);
    }
    Ok(members)
}

/// Remove a channel member.
pub fn remove_channel_member(conn: &Connection, channel_id: &str, agent_id: &str) -> Result<(), DbError> {
    conn.execute(
        "DELETE FROM channel_members WHERE channel_id = ?1 AND agent_id = ?2",
        params![channel_id, agent_id],
    )?;
    Ok(())
}

// ===========================================================================
// Thread queries
// ===========================================================================

/// Thread row from the database.
#[derive(Debug, Clone)]
pub struct ThreadRow {
    pub id: String,
    pub agent_id: String,
    pub title: String,
    pub session_id: Option<String>,
    pub message_count: i64,
    pub jsonl_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl ThreadRow {
    fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            agent_id: row.get("agent_id")?,
            title: row.get("title")?,
            session_id: row.get("session_id")?,
            message_count: row.get("message_count")?,
            jsonl_path: row.get("jsonl_path")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// Insert a thread into the database.
pub fn insert_thread(conn: &Connection, thread: &ThreadRow) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR REPLACE INTO threads (id, agent_id, title, session_id, message_count, jsonl_path, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            thread.id,
            thread.agent_id,
            thread.title,
            thread.session_id,
            thread.message_count,
            thread.jsonl_path,
            thread.created_at,
            thread.updated_at,
        ],
    )?;
    Ok(())
}

/// List all threads for an agent, ordered by updated_at desc.
pub fn list_threads_by_agent(conn: &Connection, agent_id: &str) -> Result<Vec<ThreadRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM threads WHERE agent_id = ?1 ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map(params![agent_id], ThreadRow::from_row)?;
    let mut threads = Vec::new();
    for row in rows {
        threads.push(row?);
    }
    Ok(threads)
}

/// List all threads, ordered by updated_at desc.
pub fn list_all_threads(conn: &Connection) -> Result<Vec<ThreadRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM threads ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map([], ThreadRow::from_row)?;
    let mut threads = Vec::new();
    for row in rows {
        threads.push(row?);
    }
    Ok(threads)
}

/// Get a single thread by ID.
pub fn get_thread(conn: &Connection, thread_id: &str) -> Result<Option<ThreadRow>, DbError> {
    let mut stmt = conn.prepare("SELECT * FROM threads WHERE id = ?1")?;
    let mut rows = stmt.query_map(params![thread_id], ThreadRow::from_row)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Update thread metadata (message_count, updated_at).
pub fn update_thread_meta(
    conn: &Connection,
    thread_id: &str,
    message_count: i64,
    updated_at: &str,
) -> Result<(), DbError> {
    conn.execute(
        "UPDATE threads SET message_count = ?1, updated_at = ?2 WHERE id = ?3",
        params![message_count, updated_at, thread_id],
    )?;
    Ok(())
}

/// Delete a thread from the database.
pub fn delete_thread(conn: &Connection, thread_id: &str) -> Result<(), DbError> {
    conn.execute("DELETE FROM threads WHERE id = ?1", params![thread_id])?;
    Ok(())
}

// ===========================================================================
// Task queries
// ===========================================================================

/// Task row from the database.
#[derive(Debug, Clone)]
pub struct TaskRow {
    pub id: String,
    pub title: String,
    pub status: String,
    pub assignee: Option<String>,
    pub thread_id: Option<String>,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

impl TaskRow {
    fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            title: row.get("title")?,
            status: row.get("status")?,
            assignee: row.get("assignee")?,
            thread_id: row.get("thread_id")?,
            description: row.get("description")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// Insert a task into the database.
pub fn insert_task(conn: &Connection, task: &TaskRow) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR REPLACE INTO tasks (id, title, status, assignee, thread_id, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            task.id,
            task.title,
            task.status,
            task.assignee,
            task.thread_id,
            task.description,
            task.created_at,
            task.updated_at,
        ],
    )?;
    Ok(())
}

/// List tasks with optional status filter, ordered by created_at.
pub fn list_tasks(conn: &Connection, status_filter: Option<&str>) -> Result<Vec<TaskRow>, DbError> {
    let tasks = match status_filter {
        Some(status) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM tasks WHERE status = ?1 ORDER BY created_at ASC"
            )?;
            let rows = stmt.query_map(params![status], TaskRow::from_row)?;
            rows.filter_map(|r| r.ok()).collect()
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT * FROM tasks ORDER BY created_at ASC"
            )?;
            let rows = stmt.query_map([], TaskRow::from_row)?;
            rows.filter_map(|r| r.ok()).collect()
        }
    };
    Ok(tasks)
}

/// Update task status.
pub fn update_task_status(
    conn: &Connection,
    task_id: &str,
    status: &str,
    updated_at: &str,
) -> Result<(), DbError> {
    conn.execute(
        "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status, updated_at, task_id],
    )?;
    Ok(())
}

/// Delete a task.
pub fn delete_task(conn: &Connection, task_id: &str) -> Result<(), DbError> {
    conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id])?;
    Ok(())
}

// ===========================================================================
// Skill queries
// ===========================================================================

/// Skill row from the database.
#[derive(Debug, Clone)]
pub struct SkillRow {
    pub id: String,
    pub agent_id: String,
    pub name: String,
    pub skill_type: String,
    pub status: String,
    pub config_json: String,
    pub created_at: String,
    pub updated_at: String,
}

impl SkillRow {
    fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            agent_id: row.get("agent_id")?,
            name: row.get("name")?,
            skill_type: row.get("skill_type")?,
            status: row.get("status")?,
            config_json: row.get("config_json")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// Insert a skill into the database.
pub fn insert_skill(conn: &Connection, skill: &SkillRow) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR REPLACE INTO skills (id, agent_id, name, skill_type, status, config_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            skill.id,
            skill.agent_id,
            skill.name,
            skill.skill_type,
            skill.status,
            skill.config_json,
            skill.created_at,
            skill.updated_at,
        ],
    )?;
    Ok(())
}

/// List all skills for an agent.
pub fn list_skills_by_agent(conn: &Connection, agent_id: &str) -> Result<Vec<SkillRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM skills WHERE agent_id = ?1 ORDER BY created_at ASC"
    )?;
    let rows = stmt.query_map(params![agent_id], SkillRow::from_row)?;
    let mut skills = Vec::new();
    for row in rows {
        skills.push(row?);
    }
    Ok(skills)
}

/// Delete a skill.
pub fn delete_skill(conn: &Connection, skill_id: &str) -> Result<(), DbError> {
    conn.execute("DELETE FROM skills WHERE id = ?1", params![skill_id])?;
    Ok(())
}

// ===========================================================================
// Activity log queries
// ===========================================================================

/// Activity log row from the database.
#[derive(Debug, Clone)]
pub struct ActivityLogRow {
    pub id: String,
    pub timestamp: String,
    pub activity_type: String,
    pub agent_id: Option<String>,
    pub workspace_id: Option<String>,
    pub summary: String,
    pub details_json: String,
}

impl ActivityLogRow {
    fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            timestamp: row.get("timestamp")?,
            activity_type: row.get("activity_type")?,
            agent_id: row.get("agent_id")?,
            workspace_id: row.get("workspace_id")?,
            summary: row.get("summary")?,
            details_json: row.get("details_json")?,
        })
    }
}

/// Insert an activity log entry into the database.
pub fn insert_activity(conn: &Connection, entry: &ActivityLogRow) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO activity_log (id, timestamp, activity_type, agent_id, workspace_id, summary, details_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            entry.id,
            entry.timestamp,
            entry.activity_type,
            entry.agent_id,
            entry.workspace_id,
            entry.summary,
            entry.details_json,
        ],
    )?;
    Ok(())
}

/// List activity log entries with optional agent filter, newest first.
pub fn list_activities(
    conn: &Connection,
    agent_id: Option<&str>,
    offset: usize,
    limit: usize,
) -> Result<Vec<ActivityLogRow>, DbError> {
    let entries = match agent_id {
        Some(aid) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM activity_log WHERE agent_id = ?1 ORDER BY timestamp DESC LIMIT ?2 OFFSET ?3"
            )?;
            let rows = stmt.query_map(params![aid, limit as i64, offset as i64], ActivityLogRow::from_row)?;
            rows.filter_map(|r| r.ok()).collect()
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT * FROM activity_log ORDER BY timestamp DESC LIMIT ?1 OFFSET ?2"
            )?;
            let rows = stmt.query_map(params![limit as i64, offset as i64], ActivityLogRow::from_row)?;
            rows.filter_map(|r| r.ok()).collect()
        }
    };
    Ok(entries)
}

/// Count total activity log entries with optional agent filter.
pub fn count_activities(conn: &Connection, agent_id: Option<&str>) -> Result<i64, DbError> {
    let count = match agent_id {
        Some(aid) => {
            conn.query_row(
                "SELECT COUNT(*) FROM activity_log WHERE agent_id = ?1",
                params![aid],
                |row| row.get(0),
            )?
        }
        None => {
            conn.query_row(
                "SELECT COUNT(*) FROM activity_log",
                [],
                |row| row.get(0),
            )?
        }
    };
    Ok(count)
}

// ===========================================================================
// Migration helpers
// ===========================================================================

/// Check if a table has any rows (used to detect if migration is needed).
pub fn table_row_count(conn: &Connection, table: &str) -> Result<i64, DbError> {
    let count: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM {}", table),
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db;

    fn test_conn() -> Connection {
        let dir = tempfile::tempdir().unwrap();
        db::init_database(dir.path()).unwrap()
    }

    #[test]
    fn test_insert_and_list_agents() {
        let conn = test_conn();

        insert_agent(&conn, &AgentRow {
            id: "default".to_string(),
            name: "Default".to_string(),
            emoji: "robot".to_string(),
            avatar_path: None,
            enabled: true,
            runtime_type: "claude-code".to_string(),
            description: "Default agent".to_string(),
            created_at: "2026-04-10T12:00:00Z".to_string(),
            updated_at: "2026-04-10T12:00:00Z".to_string(),
        }).unwrap();

        let agents = list_agents(&conn).unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "default");
    }

    #[test]
    fn test_insert_and_list_threads() {
        let conn = test_conn();

        insert_thread(&conn, &ThreadRow {
            id: "thread-1".to_string(),
            agent_id: "default".to_string(),
            title: "Test Thread".to_string(),
            session_id: Some("sess-1".to_string()),
            message_count: 3,
            jsonl_path: Some("agents/default/conversations/threads/thread-1.jsonl".to_string()),
            created_at: "2026-04-10T12:00:00Z".to_string(),
            updated_at: "2026-04-10T12:01:00Z".to_string(),
        }).unwrap();

        let threads = list_threads_by_agent(&conn, "default").unwrap();
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].title, "Test Thread");
        assert_eq!(threads[0].message_count, 3);
    }

    #[test]
    fn test_insert_and_list_tasks() {
        let conn = test_conn();

        for i in 0..5 {
            let status = if i < 3 { "todo" } else if i < 4 { "in_progress" } else { "done" };
            insert_task(&conn, &TaskRow {
                id: format!("task-{}", i),
                title: format!("Task {}", i),
                status: status.to_string(),
                assignee: None,
                thread_id: None,
                description: String::new(),
                created_at: "2026-04-10T12:00:00Z".to_string(),
                updated_at: "2026-04-10T12:00:00Z".to_string(),
            }).unwrap();
        }

        let all = list_tasks(&conn, None).unwrap();
        assert_eq!(all.len(), 5);

        let todos = list_tasks(&conn, Some("todo")).unwrap();
        assert_eq!(todos.len(), 3);
    }

    #[test]
    fn test_insert_and_list_activities() {
        let conn = test_conn();

        for i in 0..5 {
            insert_activity(&conn, &ActivityLogRow {
                id: format!("act-{}", i),
                timestamp: format!("2026-04-10T12:0{}:00Z", i),
                activity_type: "agent_created".to_string(),
                agent_id: if i % 2 == 0 { Some("a".to_string()) } else { Some("b".to_string()) },
                workspace_id: None,
                summary: format!("Activity {}", i),
                details_json: "{}".to_string(),
            }).unwrap();
        }

        let all = list_activities(&conn, None, 0, 100).unwrap();
        assert_eq!(all.len(), 5);

        let filtered = list_activities(&conn, Some("a"), 0, 100).unwrap();
        assert_eq!(filtered.len(), 3);

        // Newest first
        assert_eq!(filtered[0].id, "act-4");
    }
}
