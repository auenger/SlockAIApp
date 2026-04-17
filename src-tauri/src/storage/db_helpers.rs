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
    pub connection_mode: String,
    pub remote_connection_id: Option<String>,
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
            connection_mode: row.get("connection_mode").unwrap_or_else(|_| "local".to_string()),
            remote_connection_id: row.get("remote_connection_id").unwrap_or(None),
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// Insert an agent into the database.
pub fn insert_agent(conn: &Connection, agent: &AgentRow) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR REPLACE INTO agents (id, name, emoji, avatar_path, enabled, runtime_type, description, connection_mode, remote_connection_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            agent.id,
            agent.name,
            agent.emoji,
            agent.avatar_path,
            agent.enabled as i64,
            agent.runtime_type,
            agent.description,
            agent.connection_mode,
            agent.remote_connection_id,
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

/// Task row from the database (V004 schema).
#[derive(Debug, Clone)]
pub struct TaskRow {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: i64,
    pub creator_type: String,
    pub creator_id: String,
    pub assignee_id: Option<String>,
    pub channel_id: Option<String>,
    pub thread_id: Option<String>,
    pub parent_task_id: Option<String>,
    pub execution_mode: String,
    pub source: String,
    pub source_message_id: Option<String>,
    pub result: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

impl TaskRow {
    pub fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            title: row.get("title")?,
            description: row.get("description")?,
            status: row.get("status")?,
            priority: row.get("priority")?,
            creator_type: row.get("creator_type")?,
            creator_id: row.get("creator_id")?,
            assignee_id: row.get("assignee_id")?,
            channel_id: row.get("channel_id")?,
            thread_id: row.get("thread_id")?,
            parent_task_id: row.get("parent_task_id")?,
            execution_mode: row.get("execution_mode")?,
            source: row.get("source")?,
            source_message_id: row.get("source_message_id")?,
            result: row.get("result")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            completed_at: row.get("completed_at")?,
        })
    }
}

/// Insert a task into the database.
pub fn insert_task(conn: &Connection, task: &TaskRow) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO tasks (
            id, title, description, status, priority,
            creator_type, creator_id, assignee_id, channel_id,
            thread_id, parent_task_id, execution_mode, source,
            source_message_id, result, created_at, updated_at, completed_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8, ?9,
            ?10, ?11, ?12, ?13,
            ?14, ?15, ?16, ?17, ?18
        )",
        params![
            task.id,
            task.title,
            task.description,
            task.status,
            task.priority,
            task.creator_type,
            task.creator_id,
            task.assignee_id,
            task.channel_id,
            task.thread_id,
            task.parent_task_id,
            task.execution_mode,
            task.source,
            task.source_message_id,
            task.result,
            task.created_at,
            task.updated_at,
            task.completed_at,
        ],
    )?;
    Ok(())
}

/// Get a single task by ID.
pub fn get_task(conn: &Connection, task_id: &str) -> Result<Option<TaskRow>, DbError> {
    let mut stmt = conn.prepare("SELECT * FROM tasks WHERE id = ?1")?;
    let mut rows = stmt.query_map(params![task_id], TaskRow::from_row)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// List tasks with flexible filters, ordered by created_at.
pub fn list_tasks_filtered(
    conn: &Connection,
    status_filter: Option<&str>,
    channel_id: Option<&str>,
    assignee_id: Option<&str>,
    parent_task_id: Option<&str>,
) -> Result<Vec<TaskRow>, DbError> {
    let mut sql = String::from("SELECT * FROM tasks WHERE 1=1");
    let mut param_idx = 1u32;
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(status) = status_filter {
        sql.push_str(&format!(" AND status = ?{}", param_idx));
        param_values.push(Box::new(status.to_string()));
        param_idx += 1;
    }
    if let Some(cid) = channel_id {
        sql.push_str(&format!(" AND channel_id = ?{}", param_idx));
        param_values.push(Box::new(cid.to_string()));
        param_idx += 1;
    }
    if let Some(aid) = assignee_id {
        sql.push_str(&format!(" AND assignee_id = ?{}", param_idx));
        param_values.push(Box::new(aid.to_string()));
        param_idx += 1;
    }
    if let Some(pid) = parent_task_id {
        sql.push_str(&format!(" AND parent_task_id = ?{}", param_idx));
        param_values.push(Box::new(pid.to_string()));
        param_idx += 1;
    }

    sql.push_str(" ORDER BY created_at ASC");

    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(params.as_slice(), TaskRow::from_row)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// List tasks with optional status filter (simplified), ordered by created_at.
pub fn list_tasks(conn: &Connection, status_filter: Option<&str>) -> Result<Vec<TaskRow>, DbError> {
    list_tasks_filtered(conn, status_filter, None, None, None)
}

/// Update a task's mutable fields.
pub fn update_task(
    conn: &Connection,
    task_id: &str,
    title: Option<&str>,
    description: Option<&str>,
    status: Option<&str>,
    priority: Option<i64>,
    assignee_id: Option<Option<&str>>,
    execution_mode: Option<&str>,
    result: Option<Option<&str>>,
    updated_at: &str,
) -> Result<(), DbError> {
    let mut set_clauses = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut param_idx = 1u32;

    if let Some(v) = title {
        set_clauses.push(format!("title = ?{}", param_idx));
        param_values.push(Box::new(v.to_string()));
        param_idx += 1;
    }
    if let Some(v) = description {
        set_clauses.push(format!("description = ?{}", param_idx));
        param_values.push(Box::new(v.to_string()));
        param_idx += 1;
    }
    if let Some(v) = status {
        set_clauses.push(format!("status = ?{}", param_idx));
        param_values.push(Box::new(v.to_string()));
        param_idx += 1;
    }
    if let Some(v) = priority {
        set_clauses.push(format!("priority = ?{}", param_idx));
        param_values.push(Box::new(v));
        param_idx += 1;
    }
    if let Some(v) = assignee_id {
        set_clauses.push(format!("assignee_id = ?{}", param_idx));
        param_values.push(Box::new(v.map(|s| s.to_string())));
        param_idx += 1;
    }
    if let Some(v) = execution_mode {
        set_clauses.push(format!("execution_mode = ?{}", param_idx));
        param_values.push(Box::new(v.to_string()));
        param_idx += 1;
    }
    if let Some(v) = result {
        set_clauses.push(format!("result = ?{}", param_idx));
        param_values.push(Box::new(v.map(|s| s.to_string())));
        param_idx += 1;
    }

    if set_clauses.is_empty() {
        return Ok(());
    }

    // always update updated_at
    set_clauses.push(format!("updated_at = ?{}", param_idx));
    param_values.push(Box::new(updated_at.to_string()));
    param_idx += 1;

    // WHERE id = ?
    let where_param = format!("id = ?{}", param_idx);
    param_values.push(Box::new(task_id.to_string()));

    let sql = format!("UPDATE tasks SET {} WHERE {}", set_clauses.join(", "), where_param);
    let params: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, params.as_slice())?;
    Ok(())
}

/// Update task status (convenience wrapper).
pub fn update_task_status(
    conn: &Connection,
    task_id: &str,
    status: &str,
    updated_at: &str,
) -> Result<(), DbError> {
    update_task(conn, task_id, None, None, Some(status), None, None, None, None, updated_at)
}

/// Update task status with completed_at timestamp.
pub fn update_task_status_with_completed(
    conn: &Connection,
    task_id: &str,
    status: &str,
    completed_at: Option<&str>,
    updated_at: &str,
) -> Result<(), DbError> {
    let mut set_parts = vec![
        "status = ?1".to_string(),
        format!("updated_at = ?2"),
    ];
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
        Box::new(status.to_string()),
        Box::new(updated_at.to_string()),
    ];
    let mut param_idx = 3u32;

    if let Some(ca) = completed_at {
        set_parts.push(format!("completed_at = ?{}", param_idx));
        param_values.push(Box::new(ca.to_string()));
        param_idx += 1;
    }

    let where_clause = format!("id = ?{}", param_idx);
    param_values.push(Box::new(task_id.to_string()));

    let sql = format!("UPDATE tasks SET {} WHERE {}", set_parts.join(", "), where_clause);
    let params: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, params.as_slice())?;
    Ok(())
}

/// Assign/reassign a task to an agent.
pub fn assign_task(
    conn: &Connection,
    task_id: &str,
    assignee_id: Option<&str>,
    updated_at: &str,
) -> Result<(), DbError> {
    conn.execute(
        "UPDATE tasks SET assignee_id = ?1, updated_at = ?2 WHERE id = ?3",
        params![assignee_id, updated_at, task_id],
    )?;
    Ok(())
}

/// Delete a task.
pub fn delete_task(conn: &Connection, task_id: &str) -> Result<(), DbError> {
    conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id])?;
    Ok(())
}

/// Count child tasks for a parent task.
pub fn count_child_tasks(conn: &Connection, parent_task_id: &str) -> Result<i64, DbError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tasks WHERE parent_task_id = ?1",
        params![parent_task_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Count dependencies for a task.
pub fn count_dependencies(conn: &Connection, task_id: &str) -> Result<i64, DbError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM task_dependencies WHERE task_id = ?1",
        params![task_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

// ===========================================================================
// Task dependency queries
// ===========================================================================

/// Task dependency row from the database.
#[derive(Debug, Clone)]
pub struct TaskDependencyRow {
    pub task_id: String,
    pub depends_on_id: String,
    pub created_at: String,
}

impl TaskDependencyRow {
    pub fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            task_id: row.get("task_id")?,
            depends_on_id: row.get("depends_on_id")?,
            created_at: row.get("created_at")?,
        })
    }
}

/// Add a task dependency.
pub fn add_task_dependency(
    conn: &Connection,
    task_id: &str,
    depends_on_id: &str,
) -> Result<(), DbError> {
    let now = chrono_now_iso();
    conn.execute(
        "INSERT OR IGNORE INTO task_dependencies (task_id, depends_on_id, created_at)
         VALUES (?1, ?2, ?3)",
        params![task_id, depends_on_id, now],
    )?;
    Ok(())
}

/// Remove a task dependency.
pub fn remove_task_dependency(
    conn: &Connection,
    task_id: &str,
    depends_on_id: &str,
) -> Result<(), DbError> {
    conn.execute(
        "DELETE FROM task_dependencies WHERE task_id = ?1 AND depends_on_id = ?2",
        params![task_id, depends_on_id],
    )?;
    Ok(())
}

/// Get all dependencies for a task.
pub fn get_task_dependencies(conn: &Connection, task_id: &str) -> Result<Vec<TaskDependencyRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM task_dependencies WHERE task_id = ?1"
    )?;
    let rows = stmt.query_map(params![task_id], TaskDependencyRow::from_row)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Get all tasks that depend on a given task.
pub fn get_dependent_tasks(conn: &Connection, depends_on_id: &str) -> Result<Vec<TaskDependencyRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM task_dependencies WHERE depends_on_id = ?1"
    )?;
    let rows = stmt.query_map(params![depends_on_id], TaskDependencyRow::from_row)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Check if adding a dependency would create a cycle (BFS from depends_on_id to task_id).
pub fn would_create_cycle(conn: &Connection, task_id: &str, depends_on_id: &str) -> Result<bool, DbError> {
    use std::collections::{HashSet, VecDeque};
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(depends_on_id.to_string());

    while let Some(current) = queue.pop_front() {
        if current == task_id {
            return Ok(true); // cycle detected
        }
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());

        // Find all tasks that current depends on
        let deps = get_task_dependencies(conn, &current)?;
        for dep in deps {
            queue.push_back(dep.depends_on_id);
        }
    }
    Ok(false)
}

// ===========================================================================
// Task history queries
// ===========================================================================

/// Task history row from the database.
#[derive(Debug, Clone)]
pub struct TaskHistoryRow {
    pub id: i64,
    pub task_id: String,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: String,
    pub changed_at: String,
}

impl TaskHistoryRow {
    pub fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            task_id: row.get("task_id")?,
            field: row.get("field")?,
            old_value: row.get("old_value")?,
            new_value: row.get("new_value")?,
            changed_by: row.get("changed_by")?,
            changed_at: row.get("changed_at")?,
        })
    }
}

/// Record a task history entry.
pub fn insert_task_history(
    conn: &Connection,
    task_id: &str,
    field: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
    changed_by: &str,
) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO task_history (task_id, field, old_value, new_value, changed_by)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![task_id, field, old_value, new_value, changed_by],
    )?;
    Ok(())
}

/// Get history entries for a task, newest first.
pub fn get_task_history(conn: &Connection, task_id: &str) -> Result<Vec<TaskHistoryRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM task_history WHERE task_id = ?1 ORDER BY changed_at DESC"
    )?;
    let rows = stmt.query_map(params![task_id], TaskHistoryRow::from_row)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

// ===========================================================================
// Utility
// ===========================================================================

/// Get current ISO 8601 timestamp (used for DB timestamps).
pub fn chrono_now_iso() -> String {
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
// Remote connection queries
// ===========================================================================

/// Remote connection row from the database.
#[derive(Debug, Clone)]
pub struct RemoteConnectionRow {
    pub id: String,
    pub name: String,
    pub endpoint_url: String,
    pub auth_type: String,
    pub status: String,
    pub cached_agent_card: Option<String>,
    pub last_health_check_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl RemoteConnectionRow {
    pub fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            endpoint_url: row.get("endpoint_url")?,
            auth_type: row.get("auth_type")?,
            status: row.get("status")?,
            cached_agent_card: row.get("cached_agent_card")?,
            last_health_check_at: row.get("last_health_check_at")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// Insert a remote connection into the database.
pub fn insert_remote_connection(conn: &Connection, rc: &RemoteConnectionRow) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO remote_connections (id, name, endpoint_url, auth_type, status, cached_agent_card, last_health_check_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            rc.id,
            rc.name,
            rc.endpoint_url,
            rc.auth_type,
            rc.status,
            rc.cached_agent_card,
            rc.last_health_check_at,
            rc.created_at,
            rc.updated_at,
        ],
    )?;
    Ok(())
}

/// List all remote connections.
pub fn list_remote_connections(conn: &Connection) -> Result<Vec<RemoteConnectionRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM remote_connections ORDER BY name ASC"
    )?;
    let rows = stmt.query_map([], RemoteConnectionRow::from_row)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Get a single remote connection by ID.
pub fn get_remote_connection(conn: &Connection, id: &str) -> Result<Option<RemoteConnectionRow>, DbError> {
    let mut stmt = conn.prepare("SELECT * FROM remote_connections WHERE id = ?1")?;
    let mut rows = stmt.query_map(params![id], RemoteConnectionRow::from_row)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Update a remote connection's mutable fields.
pub fn update_remote_connection(
    conn: &Connection,
    id: &str,
    name: Option<&str>,
    endpoint_url: Option<&str>,
    auth_type: Option<&str>,
    status: Option<&str>,
    cached_agent_card: Option<Option<&str>>,
    last_health_check_at: Option<Option<&str>>,
    updated_at: &str,
) -> Result<(), DbError> {
    let mut set_clauses = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut param_idx = 1u32;

    if let Some(v) = name {
        set_clauses.push(format!("name = ?{}", param_idx));
        param_values.push(Box::new(v.to_string()));
        param_idx += 1;
    }
    if let Some(v) = endpoint_url {
        set_clauses.push(format!("endpoint_url = ?{}", param_idx));
        param_values.push(Box::new(v.to_string()));
        param_idx += 1;
    }
    if let Some(v) = auth_type {
        set_clauses.push(format!("auth_type = ?{}", param_idx));
        param_values.push(Box::new(v.to_string()));
        param_idx += 1;
    }
    if let Some(v) = status {
        set_clauses.push(format!("status = ?{}", param_idx));
        param_values.push(Box::new(v.to_string()));
        param_idx += 1;
    }
    if let Some(v) = cached_agent_card {
        set_clauses.push(format!("cached_agent_card = ?{}", param_idx));
        param_values.push(Box::new(v.map(|s| s.to_string())));
        param_idx += 1;
    }
    if let Some(v) = last_health_check_at {
        set_clauses.push(format!("last_health_check_at = ?{}", param_idx));
        param_values.push(Box::new(v.map(|s| s.to_string())));
        param_idx += 1;
    }

    if set_clauses.is_empty() {
        return Ok(());
    }

    set_clauses.push(format!("updated_at = ?{}", param_idx));
    param_values.push(Box::new(updated_at.to_string()));
    param_idx += 1;

    let where_clause = format!("id = ?{}", param_idx);
    param_values.push(Box::new(id.to_string()));

    let sql = format!("UPDATE remote_connections SET {} WHERE {}", set_clauses.join(", "), where_clause);
    let params: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, params.as_slice())?;
    Ok(())
}

/// Delete a remote connection.
pub fn delete_remote_connection(conn: &Connection, id: &str) -> Result<(), DbError> {
    conn.execute("DELETE FROM remote_connections WHERE id = ?1", params![id])?;
    Ok(())
}

// ===========================================================================
// Remote agent queries
// ===========================================================================

/// List all remote agents (remote_connection_id IS NOT NULL) from the database.
pub fn list_remote_agents(conn: &Connection) -> Result<Vec<AgentRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM agents WHERE remote_connection_id IS NOT NULL ORDER BY name ASC"
    )?;
    let rows = stmt.query_map([], AgentRow::from_row)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// List all remote agents for a specific connection.
pub fn list_remote_agents_by_connection(
    conn: &Connection,
    connection_id: &str,
) -> Result<Vec<AgentRow>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT * FROM agents WHERE remote_connection_id = ?1 ORDER BY name ASC"
    )?;
    let rows = stmt.query_map(params![connection_id], AgentRow::from_row)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Get a remote agent by its connection-scoped ID (format: "remote:{connection_id}:{agent_name}").
pub fn get_remote_agent(
    conn: &Connection,
    agent_id: &str,
) -> Result<Option<AgentRow>, DbError> {
    get_agent(conn, agent_id)
}

/// Upsert a remote agent: insert if not exists, update name/emoji/description if exists.
pub fn upsert_remote_agent(conn: &Connection, agent: &AgentRow) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO agents (id, name, emoji, avatar_path, enabled, runtime_type, description, connection_mode, remote_connection_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            emoji = excluded.emoji,
            description = excluded.description,
            enabled = excluded.enabled,
            updated_at = excluded.updated_at",
        params![
            agent.id,
            agent.name,
            agent.emoji,
            agent.avatar_path,
            agent.enabled as i64,
            agent.runtime_type,
            agent.description,
            agent.connection_mode,
            agent.remote_connection_id,
            agent.created_at,
            agent.updated_at,
        ],
    )?;
    Ok(())
}

/// Delete all remote agents for a specific connection (cascade on connection delete).
pub fn delete_remote_agents_by_connection(
    conn: &Connection,
    connection_id: &str,
) -> Result<usize, DbError> {
    let count = conn.execute(
        "DELETE FROM agents WHERE remote_connection_id = ?1",
        params![connection_id],
    )?;
    Ok(count)
}

/// Mark all remote agents for a connection as disabled (offline).
pub fn disable_remote_agents_by_connection(
    conn: &Connection,
    connection_id: &str,
) -> Result<usize, DbError> {
    let now = chrono_now_iso();
    let count = conn.execute(
        "UPDATE agents SET enabled = 0, updated_at = ?1 WHERE remote_connection_id = ?2",
        params![now, connection_id],
    )?;
    Ok(count)
}

/// Enable all remote agents for a connection (back online).
pub fn enable_remote_agents_by_connection(
    conn: &Connection,
    connection_id: &str,
) -> Result<usize, DbError> {
    let now = chrono_now_iso();
    let count = conn.execute(
        "UPDATE agents SET enabled = 1, updated_at = ?1 WHERE remote_connection_id = ?2",
        params![now, connection_id],
    )?;
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
            connection_mode: "local".to_string(),
            remote_connection_id: None,
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
                priority: 3,
                creator_type: "user".to_string(),
                creator_id: "user".to_string(),
                assignee_id: None,
                channel_id: None,
                thread_id: None,
                parent_task_id: None,
                execution_mode: "realtime".to_string(),
                source: "manual".to_string(),
                source_message_id: None,
                description: String::new(),
                result: None,
                created_at: "2026-04-10T12:00:00Z".to_string(),
                updated_at: "2026-04-10T12:00:00Z".to_string(),
                completed_at: None,
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
