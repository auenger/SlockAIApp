//! Tauri IPC commands for Task operations.
//!
//! Provides CRUD operations for Tasks, dependency management,
//! status transitions, and history tracking.

use crate::storage::db_helpers;
use crate::commands::AppState;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

// ---------------------------------------------------------------------------
// Types shared across commands
// ---------------------------------------------------------------------------

/// Task returned to the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInfo {
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
    // Joined / computed fields
    pub child_task_count: i64,
    pub dependency_count: i64,
}

impl TaskInfo {
    pub fn from_row(row: &db_helpers::TaskRow, child_count: i64, dep_count: i64) -> Self {
        Self {
            id: row.id.clone(),
            title: row.title.clone(),
            description: row.description.clone(),
            status: row.status.clone(),
            priority: row.priority,
            creator_type: row.creator_type.clone(),
            creator_id: row.creator_id.clone(),
            assignee_id: row.assignee_id.clone(),
            channel_id: row.channel_id.clone(),
            thread_id: row.thread_id.clone(),
            parent_task_id: row.parent_task_id.clone(),
            execution_mode: row.execution_mode.clone(),
            source: row.source.clone(),
            source_message_id: row.source_message_id.clone(),
            result: row.result.clone(),
            created_at: row.created_at.clone(),
            updated_at: row.updated_at.clone(),
            completed_at: row.completed_at.clone(),
            child_task_count: child_count,
            dependency_count: dep_count,
        }
    }
}

/// Task history entry returned to the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskHistoryInfo {
    pub id: i64,
    pub task_id: String,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: String,
    pub changed_at: String,
}

impl TaskHistoryInfo {
    fn from_row(row: &db_helpers::TaskHistoryRow) -> Self {
        Self {
            id: row.id,
            task_id: row.task_id.clone(),
            field: row.field.clone(),
            old_value: row.old_value.clone(),
            new_value: row.new_value.clone(),
            changed_by: row.changed_by.clone(),
            changed_at: row.changed_at.clone(),
        }
    }
}

/// Task dependency info returned to the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDependencyInfo {
    pub task_id: String,
    pub depends_on_id: String,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Request to create a new Task.
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_priority")]
    pub priority: i64,
    pub creator_id: String,
    #[serde(default)]
    pub creator_type: String,
    pub assignee_id: Option<String>,
    pub channel_id: Option<String>,
    pub thread_id: Option<String>,
    pub parent_task_id: Option<String>,
    #[serde(default = "default_execution_mode")]
    pub execution_mode: String,
    #[serde(default = "default_source")]
    pub source: String,
    pub source_message_id: Option<String>,
}

fn default_priority() -> i64 {
    3
}
fn default_execution_mode() -> String {
    "realtime".to_string()
}
fn default_source() -> String {
    "manual".to_string()
}

/// Request to update an existing Task.
#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i64>,
    pub assignee_id: Option<Option<String>>,
    pub execution_mode: Option<String>,
    pub result: Option<Option<String>>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Generate a new UUID for a task.
pub fn generate_task_id_helper() -> String {
    // Use a simple random UUID v4 (without pulling in uuid crate).
    // Format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();

    // Combine multiple entropy sources
    let a = secs.wrapping_mul(nanos as u64);
    let b = secs ^ (nanos as u64).wrapping_shl(32);
    let c = (a ^ b).wrapping_mul(0x2545_f491_4f6c_dd1d);
    let d = c.wrapping_shr(32) | (c.wrapping_shl(32));

    // Format as UUID with version/variant bits
    let mut bytes = [0u8; 16];
    bytes[0..8].copy_from_slice(&a.to_le_bytes());
    bytes[8..16].copy_from_slice(&d.to_le_bytes());
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // variant

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

/// Validate task status against allowed values.
fn validate_status(status: &str) -> Result<(), String> {
    match status {
        "todo" | "in_progress" | "in_review" | "done" | "blocked" | "cancelled" => Ok(()),
        _ => Err(format!(
            "invalid status '{}'. Allowed: todo, in_progress, in_review, done, blocked, cancelled",
            status
        )),
    }
}

/// Validate priority against allowed range.
fn validate_priority(priority: i64) -> Result<(), String> {
    if (1..=5).contains(&priority) {
        Ok(())
    } else {
        Err(format!(
            "invalid priority '{}'. Must be 1-5",
            priority
        ))
    }
}

/// Convert a TaskRow to TaskInfo with computed fields.
fn task_row_to_info(conn: &rusqlite::Connection, row: &db_helpers::TaskRow) -> Result<TaskInfo, String> {
    let child_count = db_helpers::count_child_tasks(conn, &row.id)
        .map_err(|e| format!("count children failed: {e}"))?;
    let dep_count = db_helpers::count_dependencies(conn, &row.id)
        .map_err(|e| format!("count deps failed: {e}"))?;
    Ok(TaskInfo::from_row(row, child_count, dep_count))
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Create a new Task.
#[tauri::command]
pub fn create_task(
    state: tauri::State<'_, AppState>,
    request: CreateTaskRequest,
) -> Result<TaskInfo, String> {
    validate_priority(request.priority)?;

    let creator_type = if request.creator_type.is_empty() {
        "user".to_string()
    } else {
        request.creator_type.clone()
    };

    let now = db_helpers::chrono_now_iso();
    let task_id = generate_task_id_helper();

    let task_row = db_helpers::TaskRow {
        id: task_id,
        title: request.title,
        description: request.description,
        status: "todo".to_string(),
        priority: request.priority,
        creator_type,
        creator_id: request.creator_id,
        assignee_id: request.assignee_id,
        channel_id: request.channel_id,
        thread_id: request.thread_id,
        parent_task_id: request.parent_task_id,
        execution_mode: request.execution_mode,
        source: request.source,
        source_message_id: request.source_message_id,
        result: None,
        created_at: now.clone(),
        updated_at: now,
        completed_at: None,
    };

    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    db_helpers::insert_task(&conn, &task_row)
        .map_err(|e| format!("insert task failed: {e}"))?;

    // Record creation in history
    db_helpers::insert_task_history(&conn, &task_row.id, "status", None, Some("todo"), &format!("{}:{}", task_row.creator_type, task_row.creator_id))
        .map_err(|e| format!("insert history failed: {e}"))?;

    let info = task_row_to_info(&conn, &task_row)?;

    log::info!("[Task] Created task '{}' ({})", info.title, info.id);
    Ok(info)
}

/// List tasks with optional filters.
#[tauri::command]
pub fn list_tasks(
    state: tauri::State<'_, AppState>,
    status_filter: Option<String>,
    channel_id: Option<String>,
    assignee_id: Option<String>,
    parent_task_id: Option<String>,
) -> Result<Vec<TaskInfo>, String> {
    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let rows = db_helpers::list_tasks_filtered(
        &conn,
        status_filter.as_deref(),
        channel_id.as_deref(),
        assignee_id.as_deref(),
        parent_task_id.as_deref(),
    )
    .map_err(|e| format!("list tasks failed: {e}"))?;

    let mut infos = Vec::new();
    for row in &rows {
        infos.push(task_row_to_info(&conn, row)?);
    }
    Ok(infos)
}

/// Get a single task by ID.
#[tauri::command]
pub fn get_task(
    state: tauri::State<'_, AppState>,
    task_id: String,
) -> Result<TaskInfo, String> {
    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let row = db_helpers::get_task(&conn, &task_id)
        .map_err(|e| format!("get task failed: {e}"))?
        .ok_or_else(|| format!("task not found: {task_id}"))?;

    task_row_to_info(&conn, &row)
}

/// Update an existing task.
#[tauri::command]
pub fn update_task(
    state: tauri::State<'_, AppState>,
    task_id: String,
    request: UpdateTaskRequest,
) -> Result<TaskInfo, String> {
    if let Some(ref status) = request.status {
        validate_status(status)?;
    }
    if let Some(priority) = request.priority {
        validate_priority(priority)?;
    }

    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    // Fetch current task for history diffing
    let current = db_helpers::get_task(&conn, &task_id)
        .map_err(|e| format!("get task failed: {e}"))?
        .ok_or_else(|| format!("task not found: {task_id}"))?;

    let now = db_helpers::chrono_now_iso();

    // Determine completed_at
    let new_completed_at = if let Some(ref status) = request.status {
        match status.as_str() {
            "done" | "cancelled" => Some(now.clone()),
            _ => None,
        }
    } else {
        None
    };

    // Use the general update function
    db_helpers::update_task(
        &conn,
        &task_id,
        request.title.as_deref(),
        request.description.as_deref(),
        request.status.as_deref(),
        request.priority,
        request.assignee_id.as_ref().map(|o| o.as_deref()),
        request.execution_mode.as_deref(),
        request.result.as_ref().map(|o| o.as_deref()),
        &now,
    )
    .map_err(|e| format!("update task failed: {e}"))?;

    // Update completed_at if status changed to done/cancelled
    if let Some(ref status) = request.status {
        if matches!(status.as_str(), "done" | "cancelled") {
            db_helpers::update_task_status_with_completed(
                &conn, &task_id, status, new_completed_at.as_deref(), &now,
            )
            .map_err(|e| format!("update completed_at failed: {e}"))?;
        }
    }

    // Record history for changed fields
    let changed_by = format!("user:{}", current.creator_id);
    if let Some(ref new_status) = request.status {
        if *new_status != current.status {
            db_helpers::insert_task_history(
                &conn, &task_id, "status", Some(&current.status), Some(new_status), &changed_by,
            )
            .map_err(|e| format!("insert history failed: {e}"))?;
        }
    }
    if let Some(ref new_title) = request.title {
        if *new_title != current.title {
            db_helpers::insert_task_history(
                &conn, &task_id, "title", Some(&current.title), Some(new_title), &changed_by,
            )
            .map_err(|e| format!("insert history failed: {e}"))?;
        }
    }
    if let Some(ref new_desc) = request.description {
        if *new_desc != current.description {
            db_helpers::insert_task_history(
                &conn, &task_id, "description", Some(&current.description), Some(new_desc), &changed_by,
            )
            .map_err(|e| format!("insert history failed: {e}"))?;
        }
    }
    if let Some(new_priority) = request.priority {
        if new_priority != current.priority {
            db_helpers::insert_task_history(
                &conn,
                &task_id,
                "priority",
                Some(&current.priority.to_string()),
                Some(&new_priority.to_string()),
                &changed_by,
            )
            .map_err(|e| format!("insert history failed: {e}"))?;
        }
    }
    if let Some(ref new_assignee) = request.assignee_id {
        let old = current.assignee_id.as_deref().unwrap_or("");
        let new = new_assignee.as_deref().unwrap_or("");
        if old != new {
            db_helpers::insert_task_history(
                &conn, &task_id, "assignee_id", Some(old), Some(new), &changed_by,
            )
            .map_err(|e| format!("insert history failed: {e}"))?;
        }
    }

    // Fetch updated task
    let updated = db_helpers::get_task(&conn, &task_id)
        .map_err(|e| format!("get updated task failed: {e}"))?
        .ok_or_else(|| format!("task not found after update: {task_id}"))?;

    log::info!("[Task] Updated task '{}' ({})", updated.title, updated.id);
    task_row_to_info(&conn, &updated)
}

/// Delete a task by ID.
#[tauri::command]
pub fn delete_task(
    state: tauri::State<'_, AppState>,
    task_id: String,
) -> Result<(), String> {
    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    db_helpers::delete_task(&conn, &task_id)
        .map_err(|e| format!("delete task failed: {e}"))?;

    log::info!("[Task] Deleted task {}", task_id);
    Ok(())
}

/// Update only the status of a task (convenience command for drag-and-drop).
///
/// After updating, triggers cascade rules:
/// - If task becomes `done`: check parent cascade (all children done -> parent in_review)
/// - If task becomes `done`: check dependency unlock (unblock tasks that depend on this one)
/// - If task becomes `cancelled` and has parent: cascade cancel to children
#[tauri::command]
pub fn update_task_status(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    task_id: String,
    status: String,
) -> Result<TaskInfo, String> {
    validate_status(&status)?;

    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let current = db_helpers::get_task(&conn, &task_id)
        .map_err(|e| format!("get task failed: {e}"))?
        .ok_or_else(|| format!("task not found: {task_id}"))?;

    let now = db_helpers::chrono_now_iso();
    let completed_at = match status.as_str() {
        "done" | "cancelled" => Some(now.clone()),
        _ => None,
    };

    db_helpers::update_task_status_with_completed(
        &conn, &task_id, &status, completed_at.as_deref(), &now,
    )
    .map_err(|e| format!("update status failed: {e}"))?;

    // Record history
    let changed_by = format!("user:{}", current.creator_id);
    db_helpers::insert_task_history(
        &conn, &task_id, "status", Some(&current.status), Some(&status), &changed_by,
    )
    .map_err(|e| format!("insert history failed: {e}"))?;

    // --- Cascade rules ---
    match status.as_str() {
        "done" => {
            // Check parent cascade: if all siblings are done, move parent to in_review
            if let Some(ref parent_id) = current.parent_task_id {
                if let Err(e) = check_parent_cascade(&conn, parent_id, &changed_by) {
                    log::warn!("[Task] Parent cascade check failed for {}: {}", parent_id, e);
                }
            }

            // Check dependency unlock: find tasks that depend on this one
            if let Err(e) = check_dependency_unlock(&app, &conn, &task_id) {
                log::warn!("[Task] Dependency unlock check failed for {}: {}", task_id, e);
            }
        }
        "cancelled" => {
            // Cascade cancel to all child tasks
            if let Err(e) = cascade_cancel_children(&app, &conn, &task_id, &changed_by) {
                log::warn!("[Task] Cascade cancel failed for {}: {}", task_id, e);
            }
        }
        _ => {}
    }

    let updated = db_helpers::get_task(&conn, &task_id)
        .map_err(|e| format!("get updated task failed: {e}"))?
        .ok_or_else(|| format!("task not found after update: {task_id}"))?;

    log::info!(
        "[Task] Status changed: {} -> {} for task {}",
        current.status,
        status,
        task_id
    );
    task_row_to_info(&conn, &updated)
}

/// Check if all children of a parent task are done/in_review/cancelled.
/// If so, move the parent to `in_review`.
fn check_parent_cascade(
    conn: &rusqlite::Connection,
    parent_id: &str,
    changed_by: &str,
) -> Result<(), String> {
    let children = db_helpers::list_tasks_filtered(conn, None, None, None, Some(parent_id))
        .map_err(|e| format!("list children failed: {e}"))?;

    if children.is_empty() {
        return Ok(());
    }

    // Check if all children are in a "completed" state (done, in_review, or cancelled)
    let all_done = children.iter().all(|c| {
        matches!(c.status.as_str(), "done" | "in_review" | "cancelled")
    });

    if all_done {
        let parent = db_helpers::get_task(conn, parent_id)
            .map_err(|e| format!("get parent failed: {e}"))?;

        if let Some(p) = parent {
            if p.status != "in_review" && p.status != "done" && p.status != "cancelled" {
                let now = db_helpers::chrono_now_iso();
                db_helpers::update_task_status_with_completed(
                    conn, parent_id, "in_review", None, &now,
                ).map_err(|e| format!("update parent status failed: {e}"))?;

                db_helpers::insert_task_history(
                    conn, parent_id, "status", Some(&p.status), Some("in_review"),
                    &format!("system:parent-cascade"),
                ).unwrap_or_else(|e| log::warn!("[Task] history insert failed: {e}"));

                log::info!(
                    "[Task] Parent cascade: all children done, parent {} -> in_review",
                    parent_id
                );
            }
        }
    }

    Ok(())
}

/// Check tasks that depend on the completed task. If all their dependencies are done,
/// unblock them (change status from `blocked` to `todo`).
fn check_dependency_unlock(
    app: &tauri::AppHandle,
    conn: &rusqlite::Connection,
    completed_task_id: &str,
) -> Result<(), String> {
    use tauri::Emitter;

    // Find all tasks that depend on the completed task
    let dependents = db_helpers::get_dependent_tasks(conn, completed_task_id)
        .map_err(|e| format!("get dependents failed: {e}"))?;

    for dep in &dependents {
        // Only consider tasks that are currently blocked
        let dep_task = db_helpers::get_task(conn, &dep.task_id)
            .map_err(|e| format!("get dep task failed: {e}"))?;

        if let Some(task) = dep_task {
            if task.status != "blocked" {
                continue;
            }

            // Check if ALL dependencies are now done
            let all_deps = db_helpers::get_task_dependencies(conn, &task.id)
                .map_err(|e| format!("get all deps failed: {e}"))?;

            let all_met = all_deps.iter().all(|d| {
                match db_helpers::get_task(conn, &d.depends_on_id) {
                    Ok(Some(dt)) => dt.status == "done",
                    _ => false,
                }
            });

            if all_met {
                let now = db_helpers::chrono_now_iso();
                db_helpers::update_task_status_with_completed(
                    conn, &task.id, "todo", None, &now,
                ).map_err(|e| format!("unblock task failed: {e}"))?;

                db_helpers::insert_task_history(
                    conn, &task.id, "status", Some("blocked"), Some("todo"),
                    "system:dependency-met",
                ).unwrap_or_else(|e| log::warn!("[Task] history insert failed: {e}"));

                // Emit dependency-met event
                let _ = app.emit(
                    "task://dependency-met",
                    serde_json::json!({
                        "task_id": task.id,
                        "unblocked_by": completed_task_id,
                    }),
                );

                log::info!(
                    "[Task] Dependency unlock: task {} unblocked (all deps done)",
                    task.id
                );
            }
        }
    }

    Ok(())
}

/// Cascade cancel to all child tasks of a given parent.
fn cascade_cancel_children(
    app: &tauri::AppHandle,
    conn: &rusqlite::Connection,
    parent_id: &str,
    _changed_by: &str,
) -> Result<(), String> {
    use tauri::Emitter;

    let children = db_helpers::list_tasks_filtered(conn, None, None, None, Some(parent_id))
        .map_err(|e| format!("list children failed: {e}"))?;

    for child in &children {
        // Skip already cancelled/done children
        if matches!(child.status.as_str(), "cancelled" | "done") {
            continue;
        }

        let now = db_helpers::chrono_now_iso();
        db_helpers::update_task_status_with_completed(
            conn, &child.id, "cancelled", Some(&now), &now,
        ).map_err(|e| format!("cancel child failed: {e}"))?;

        db_helpers::insert_task_history(
            conn, &child.id, "status", Some(&child.status), Some("cancelled"),
            &format!("system:parent-cancelled"),
        ).unwrap_or_else(|e| log::warn!("[Task] history insert failed: {e}"));

        // Emit event
        let _ = app.emit(
            "task://status-changed",
            serde_json::json!({
                "task_id": child.id,
                "old_status": child.status,
                "new_status": "cancelled",
            }),
        );

        log::info!(
            "[Task] Cascade cancel: child {} -> cancelled (parent {} cancelled)",
            child.id,
            parent_id
        );
    }

    Ok(())
}

/// Assign or reassign a task to an agent.
#[tauri::command]
pub fn assign_task(
    state: tauri::State<'_, AppState>,
    task_id: String,
    agent_id: Option<String>,
) -> Result<TaskInfo, String> {
    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let current = db_helpers::get_task(&conn, &task_id)
        .map_err(|e| format!("get task failed: {e}"))?
        .ok_or_else(|| format!("task not found: {task_id}"))?;

    let now = db_helpers::chrono_now_iso();
    db_helpers::assign_task(&conn, &task_id, agent_id.as_deref(), &now)
        .map_err(|e| format!("assign task failed: {e}"))?;

    // Record history
    let old_assignee = current.assignee_id.as_deref().unwrap_or("");
    let new_assignee = agent_id.as_deref().unwrap_or("");
    if old_assignee != new_assignee {
        let changed_by = format!("user:{}", current.creator_id);
        db_helpers::insert_task_history(
            &conn, &task_id, "assignee_id", Some(old_assignee), Some(new_assignee), &changed_by,
        )
        .map_err(|e| format!("insert history failed: {e}"))?;
    }

    let updated = db_helpers::get_task(&conn, &task_id)
        .map_err(|e| format!("get updated task failed: {e}"))?
        .ok_or_else(|| format!("task not found after update: {task_id}"))?;

    log::info!(
        "[Task] Assigned task {} to {:?}",
        task_id,
        agent_id
    );
    task_row_to_info(&conn, &updated)
}

/// Cancel a task (sets status to 'cancelled').
#[tauri::command]
pub fn cancel_task(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    task_id: String,
) -> Result<TaskInfo, String> {
    // cancel_task is a convenience wrapper that sets status to 'cancelled'
    update_task_status(app, state, task_id, "cancelled".to_string())
}

/// Add a dependency between two tasks.
#[tauri::command]
pub fn add_task_dependency(
    state: tauri::State<'_, AppState>,
    task_id: String,
    depends_on_id: String,
) -> Result<(), String> {
    if task_id == depends_on_id {
        return Err("a task cannot depend on itself".to_string());
    }

    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    // Verify both tasks exist
    db_helpers::get_task(&conn, &task_id)
        .map_err(|e| format!("get task failed: {e}"))?
        .ok_or_else(|| format!("task not found: {task_id}"))?;

    db_helpers::get_task(&conn, &depends_on_id)
        .map_err(|e| format!("get task failed: {e}"))?
        .ok_or_else(|| format!("depends_on task not found: {depends_on_id}"))?;

    // Check for cycles
    if db_helpers::would_create_cycle(&conn, &task_id, &depends_on_id)
        .map_err(|e| format!("cycle check failed: {e}"))?
    {
        return Err("adding this dependency would create a cycle".to_string());
    }

    db_helpers::add_task_dependency(&conn, &task_id, &depends_on_id)
        .map_err(|e| format!("add dependency failed: {e}"))?;

    log::info!(
        "[Task] Added dependency: {} depends on {}",
        task_id,
        depends_on_id
    );
    Ok(())
}

/// Remove a dependency between two tasks.
#[tauri::command]
pub fn remove_task_dependency(
    state: tauri::State<'_, AppState>,
    task_id: String,
    depends_on_id: String,
) -> Result<(), String> {
    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    db_helpers::remove_task_dependency(&conn, &task_id, &depends_on_id)
        .map_err(|e| format!("remove dependency failed: {e}"))?;

    log::info!(
        "[Task] Removed dependency: {} no longer depends on {}",
        task_id,
        depends_on_id
    );
    Ok(())
}

/// Get the history entries for a task.
#[tauri::command]
pub fn get_task_history(
    state: tauri::State<'_, AppState>,
    task_id: String,
) -> Result<Vec<TaskHistoryInfo>, String> {
    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let rows = db_helpers::get_task_history(&conn, &task_id)
        .map_err(|e| format!("get task history failed: {e}"))?;

    Ok(rows.iter().map(TaskHistoryInfo::from_row).collect())
}

/// Get dependencies for a task (tasks that this task depends on).
#[tauri::command]
pub fn get_task_dependencies(
    state: tauri::State<'_, AppState>,
    task_id: String,
) -> Result<Vec<TaskDependencyInfo>, String> {
    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let rows = db_helpers::get_task_dependencies(&conn, &task_id)
        .map_err(|e| format!("get dependencies failed: {e}"))?;

    Ok(rows.iter().map(|r| TaskDependencyInfo {
        task_id: r.task_id.clone(),
        depends_on_id: r.depends_on_id.clone(),
    }).collect())
}

/// Get tasks that depend on a given task (reverse dependencies).
#[tauri::command]
pub fn get_dependent_tasks(
    state: tauri::State<'_, AppState>,
    task_id: String,
) -> Result<Vec<TaskDependencyInfo>, String> {
    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let rows = db_helpers::get_dependent_tasks(&conn, &task_id)
        .map_err(|e| format!("get dependent tasks failed: {e}"))?;

    Ok(rows.iter().map(|r| TaskDependencyInfo {
        task_id: r.task_id.clone(),
        depends_on_id: r.depends_on_id.clone(),
    }).collect())
}

/// Get child tasks of a parent task.
#[tauri::command]
pub fn get_child_tasks(
    state: tauri::State<'_, AppState>,
    parent_task_id: String,
) -> Result<Vec<TaskInfo>, String> {
    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let rows = db_helpers::list_tasks_filtered(
        &conn, None, None, None, Some(&parent_task_id),
    )
    .map_err(|e| format!("get child tasks failed: {e}"))?;

    let mut infos = Vec::new();
    for row in &rows {
        infos.push(task_row_to_info(&conn, row)?);
    }
    Ok(infos)
}

// ---------------------------------------------------------------------------
// Task execution commands (TaskEngine integration)
// ---------------------------------------------------------------------------

/// Execute a task via the TaskEngine.
///
/// Dispatches to realtime or async execution based on the task's `execution_mode`.
/// The TaskEngine handles dependency checks, agent busy state, and emits
/// task://* events for the frontend.
#[tauri::command]
pub fn execute_task(
    task_engine: tauri::State<'_, crate::task_engine::TaskEngine>,
    state: tauri::State<'_, AppState>,
    task_id: String,
) -> Result<(), String> {
    // Load task to determine execution mode
    let conn = state
        .db_conn
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let task = db_helpers::get_task(&conn, &task_id)
        .map_err(|e| format!("get task failed: {e}"))?
        .ok_or_else(|| format!("task not found: {task_id}"))?;

    let mode = task.execution_mode.clone();

    // Release the DB lock before submitting (TaskEngine will acquire it again)
    drop(conn);

    task_engine.submit(&task_id, &mode)?;

    log::info!("[Task] Execution submitted: task_id={}, mode={}", task_id, mode);
    Ok(())
}

/// Cancel a running task execution via the TaskEngine.
///
/// This is different from `cancel_task` which just sets status to 'cancelled'.
/// This command signals the TaskEngine to interrupt an active execution.
#[tauri::command]
pub fn cancel_task_execution(
    task_engine: tauri::State<'_, crate::task_engine::TaskEngine>,
    task_id: String,
) -> Result<(), String> {
    task_engine.cancel_running_task(&task_id)?;
    log::info!("[Task] Execution cancelled: task_id={}", task_id);
    Ok(())
}

/// Report task completion to the TaskEngine (called by frontend after agent responds).
#[tauri::command]
pub fn report_task_completed(
    task_engine: tauri::State<'_, crate::task_engine::TaskEngine>,
    task_id: String,
    result: String,
) -> Result<(), String> {
    task_engine.on_task_completed(&task_id, &result)?;
    log::info!("[Task] Completion reported: task_id={}, result_len={}", task_id, result.len());
    Ok(())
}

/// Report task failure to the TaskEngine.
#[tauri::command]
pub fn report_task_failed(
    task_engine: tauri::State<'_, crate::task_engine::TaskEngine>,
    task_id: String,
    error: String,
) -> Result<(), String> {
    task_engine.on_task_failed(&task_id, &error)?;
    log::info!("[Task] Failure reported: task_id={}, error={}", task_id, error);
    Ok(())
}

/// Get active task execution status from the TaskEngine.
#[tauri::command]
pub fn get_task_engine_status(
    task_engine: tauri::State<'_, crate::task_engine::TaskEngine>,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "active_tasks": task_engine.get_active_tasks_info(),
        "queue_length": task_engine.queue_length(),
    }))
}
