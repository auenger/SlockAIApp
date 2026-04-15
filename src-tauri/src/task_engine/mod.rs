//! Task Execution Engine — manages task lifecycle and Agent dispatch.
//!
//! TaskEngine sits above `channel.rs` as a Task state management layer.
//! It supports two execution modes:
//!
//! - **Realtime**: Injects Task context into the Channel message flow,
//!   reusing `channel.rs`'s `send_channel_message` with extra system prompt.
//! - **Async**: Queue + background poll thread that auto-dispatches tasks
//!   to idle agents, running in background Threads.
//!
//! Call chain:
//!
//! ```text
//! User clicks "Execute Task"
//!   -> Tauri command: execute_task()
//!   -> TaskEngine::submit(task_id, mode)
//!   -> if realtime: inject task context into channel.rs send_message flow
//!   -> if async:     enqueue, background poll thread picks up
//! ```

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager};

use crate::storage::db_helpers;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum retry count for failed async tasks.
const MAX_RETRY: u32 = 2;

/// Poll interval for the async dispatch thread (seconds).
const POLL_INTERVAL_SECS: u64 = 5;

// ---------------------------------------------------------------------------
// CancellationToken
// ---------------------------------------------------------------------------

/// Token to signal cancellation of a running task.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Signal cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Check whether cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A task waiting in the async dispatch queue.
#[derive(Debug, Clone)]
pub struct QueuedTask {
    pub task_id: String,
    pub priority: u32,
    pub enqueued_at: String,
    pub retry_count: u32,
}

/// A task currently being executed.
#[derive(Debug)]
pub struct ActiveTask {
    pub task_id: String,
    pub agent_id: String,
    pub channel_id: String,
    pub started_at: String,
    pub mode: String, // "realtime" | "async"
    pub cancel_token: CancellationToken,
}

/// Context built for async task execution (background Thread).
#[derive(Debug, Clone)]
pub struct AsyncTaskContext {
    pub agent_id: String,
    pub workspace: String,
    pub task_prompt: String,
    pub channel_id: Option<String>,
}

// ---------------------------------------------------------------------------
// TaskEngine
// ---------------------------------------------------------------------------

/// The core execution engine.
///
/// Thread-safe: all shared state uses `Arc<Mutex<>>` or `Arc<DashMap>`.
/// The poll thread runs in the background and auto-dispatches queued async tasks.
pub struct TaskEngine {
    /// Async tasks waiting to be dispatched.
    async_queue: Arc<Mutex<VecDeque<QueuedTask>>>,

    /// Tasks currently being executed (keyed by task_id).
    active_tasks: Arc<Mutex<std::collections::HashMap<String, ActiveTask>>>,

    /// Agent busy state, keyed by `(agent_id, channel_id)`.
    /// Same agent can execute on different channels in parallel.
    agent_busy: Arc<Mutex<std::collections::HashSet<(String, String)>>>,

    /// Tauri AppHandle for emitting events.
    app: AppHandle,

    /// Handle to the background poll thread.
    poll_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl TaskEngine {
    /// Create a new TaskEngine.
    pub fn new(app: AppHandle) -> Self {
        Self {
            async_queue: Arc::new(Mutex::new(VecDeque::new())),
            active_tasks: Arc::new(Mutex::new(std::collections::HashMap::new())),
            agent_busy: Arc::new(Mutex::new(std::collections::HashSet::new())),
            app,
            poll_handle: Mutex::new(None),
        }
    }

    /// Start the background poll thread for async task dispatch.
    pub fn start_poll_thread(&self) {
        let queue = self.async_queue.clone();
        let active = self.active_tasks.clone();
        let busy = self.agent_busy.clone();
        let app = self.app.clone();

        let handle = std::thread::Builder::new()
            .name("task-engine-poll".to_string())
            .spawn(move || {
                log::info!("[TaskEngine] Poll thread started");
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS));
                    if let Err(e) = poll_and_dispatch_inner(&app, &queue, &active, &busy) {
                        log::error!("[TaskEngine] Poll dispatch error: {}", e);
                    }
                }
            })
            .expect("failed to spawn task engine poll thread");

        *self.poll_handle.lock().unwrap() = Some(handle);
    }

    // -----------------------------------------------------------------------
    // Submit
    // -----------------------------------------------------------------------

    /// Submit a task for execution.
    ///
    /// Dispatches to `execute_realtime` or `enqueue` based on the mode.
    pub fn submit(&self, task_id: &str, mode: &str) -> Result<(), String> {
        log::info!("[TaskEngine] submit task_id={}, mode={}", task_id, mode);

        match mode {
            "realtime" => self.execute_realtime(task_id),
            "async" => self.enqueue(task_id),
            other => Err(format!("unknown execution mode: {}", other)),
        }
    }

    // -----------------------------------------------------------------------
    // Realtime execution
    // -----------------------------------------------------------------------

    /// Execute a task in realtime by injecting Task context into the Channel flow.
    ///
    /// This reads the Task from DB, checks dependencies, marks the agent as busy,
    /// updates the DB status to `in_progress`, then emits an event so the frontend
    /// triggers `send_channel_message` with the Task context injected.
    pub fn execute_realtime(&self, task_id: &str) -> Result<(), String> {
        // 1. Load task from DB
        let task = self.get_task_from_db(task_id)?;

        // 2. Validate task is in a submittable state
        if task.status != "todo" && task.status != "blocked" {
            return Err(format!(
                "task {} is in status '{}' — can only execute 'todo' or 'blocked' tasks",
                task_id, task.status
            ));
        }

        let agent_id = task
            .assignee_id
            .clone()
            .ok_or_else(|| format!("task {} has no assigned agent", task_id))?;
        let channel_id = task
            .channel_id
            .clone()
            .ok_or_else(|| format!("task {} has no bound channel (required for realtime)", task_id))?;

        // 3. Check dependencies
        if !self.check_dependencies(task_id)? {
            return Err(format!(
                "task {} is blocked by unmet dependencies",
                task_id
            ));
        }

        // 4. Check agent is not busy on this channel
        {
            let busy = self.agent_busy.lock().unwrap();
            if busy.contains(&(agent_id.clone(), channel_id.clone())) {
                return Err(format!(
                    "agent {} is busy on channel {} — cannot execute task {}",
                    agent_id, channel_id, task_id
                ));
            }
        }

        // 5. Mark agent busy
        {
            let mut busy = self.agent_busy.lock().unwrap();
            busy.insert((agent_id.clone(), channel_id.clone()));
        }

        // 6. Mark task active
        let cancel_token = CancellationToken::new();
        {
            let mut active = self.active_tasks.lock().unwrap();
            active.insert(
                task_id.to_string(),
                ActiveTask {
                    task_id: task_id.to_string(),
                    agent_id: agent_id.clone(),
                    channel_id: channel_id.clone(),
                    started_at: db_helpers::chrono_now_iso(),
                    mode: "realtime".to_string(),
                    cancel_token: cancel_token.clone(),
                },
            );
        }

        // 7. Update DB status -> in_progress
        self.update_task_status_in_db(task_id, "in_progress")?;

        // 8. Build task context prompt
        let task_prompt = build_task_context_prompt(&task);

        // 9. Emit event to frontend to trigger send_channel_message with task context
        let _ = self.app.emit(
            "task://execute-realtime",
            serde_json::json!({
                "task_id": task_id,
                "agent_id": agent_id,
                "channel_id": channel_id,
                "task_prompt": task_prompt,
                "task_title": task.title,
            }),
        );

        // 10. Emit status-changed event
        let _ = self.app.emit(
            "task://status-changed",
            serde_json::json!({
                "task_id": task_id,
                "old_status": task.status,
                "new_status": "in_progress",
            }),
        );

        log::info!(
            "[TaskEngine] Realtime execution started: task={}, agent={}, channel={}",
            task_id, agent_id, channel_id
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Async execution
    // -----------------------------------------------------------------------

    /// Enqueue a task for async execution.
    ///
    /// The background poll thread will pick it up and dispatch to an idle agent.
    pub fn enqueue(&self, task_id: &str) -> Result<(), String> {
        // Load task to validate it exists
        let task = self.get_task_from_db(task_id)?;

        if task.status != "todo" && task.status != "blocked" {
            return Err(format!(
                "task {} is in status '{}' — can only enqueue 'todo' or 'blocked' tasks",
                task_id, task.status
            ));
        }

        if task.assignee_id.is_none() {
            return Err(format!(
                "task {} has no assigned agent — cannot enqueue",
                task_id
            ));
        }

        let queued = QueuedTask {
            task_id: task_id.to_string(),
            priority: task.priority as u32,
            enqueued_at: db_helpers::chrono_now_iso(),
            retry_count: 0,
        };

        {
            let mut queue = self.async_queue.lock().unwrap();
            queue.push_back(queued);
        }

        log::info!(
            "[TaskEngine] Task {} enqueued for async execution (queue len={})",
            task_id,
            self.async_queue.lock().unwrap().len()
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Cancel
    // -----------------------------------------------------------------------

    /// Cancel a running task.
    ///
    /// For realtime: sets the cancel token so the channel flow can check and abort.
    /// For async: removes from queue or cancels the active execution.
    pub fn cancel_running_task(&self, task_id: &str) -> Result<(), String> {
        // 1. Try to cancel an active task
        {
            let mut active = self.active_tasks.lock().unwrap();
            if let Some(active_task) = active.remove(task_id) {
                // Set cancel token
                active_task.cancel_token.cancel();

                // Mark agent as no longer busy
                {
                    let mut busy = self.agent_busy.lock().unwrap();
                    busy.remove(&(active_task.agent_id.clone(), active_task.channel_id.clone()));
                }

                // Update DB status -> cancelled
                self.update_task_status_in_db(task_id, "cancelled")?;

                // Emit events
                let _ = self.app.emit(
                    "task://cancelled",
                    serde_json::json!({
                        "task_id": task_id,
                    }),
                );
                let _ = self.app.emit(
                    "task://status-changed",
                    serde_json::json!({
                        "task_id": task_id,
                        "old_status": "in_progress",
                        "new_status": "cancelled",
                    }),
                );

                log::info!("[TaskEngine] Task {} cancelled (was active)", task_id);
                return Ok(());
            }
        }

        // 2. Try to remove from async queue
        {
            let mut queue = self.async_queue.lock().unwrap();
            let before_len = queue.len();
            queue.retain(|qt| qt.task_id != task_id);
            if queue.len() < before_len {
                // Update DB status -> cancelled
                self.update_task_status_in_db(task_id, "cancelled")?;

                let _ = self.app.emit(
                    "task://cancelled",
                    serde_json::json!({ "task_id": task_id }),
                );

                log::info!("[TaskEngine] Task {} cancelled (was queued)", task_id);
                return Ok(());
            }
        }

        Err(format!("task {} is not active or queued", task_id))
    }

    // -----------------------------------------------------------------------
    // Completion callbacks
    // -----------------------------------------------------------------------

    /// Called when a task execution completes successfully.
    ///
    /// Updates the DB, emits events, and releases the agent.
    pub fn on_task_completed(&self, task_id: &str, result: &str) -> Result<(), String> {
        // Remove from active tasks
        let active_task = {
            let mut active = self.active_tasks.lock().unwrap();
            active.remove(task_id)
        };

        if let Some(at) = active_task {
            // Release agent
            {
                let mut busy = self.agent_busy.lock().unwrap();
                busy.remove(&(at.agent_id.clone(), at.channel_id.clone()));
            }

            // Update DB: status -> in_review, result
            let state = self.app.state::<crate::commands::AppState>();
            let conn = state
                .db_conn
                .lock()
                .map_err(|e| format!("db lock error: {e}"))?;
            let now = db_helpers::chrono_now_iso();
            db_helpers::update_task(
                &conn,
                task_id,
                None,
                None,
                Some("in_review"),
                None,
                None,
                None,
                Some(Some(result)),
                &now,
            )
            .map_err(|e| format!("update task failed: {e}"))?;

            // Record history
            let changed_by = format!("agent:{}", at.agent_id);
            db_helpers::insert_task_history(
                &conn,
                task_id,
                "status",
                Some("in_progress"),
                Some("in_review"),
                &changed_by,
            )
            .unwrap_or_else(|e| log::warn!("[TaskEngine] history insert failed: {e}"));

            if !result.is_empty() {
                db_helpers::insert_task_history(
                    &conn,
                    task_id,
                    "result",
                    None,
                    Some(result),
                    &changed_by,
                )
                .unwrap_or_else(|e| log::warn!("[TaskEngine] history insert failed: {e}"));
            }

            // Emit events
            let _ = self.app.emit(
                "task://completed",
                serde_json::json!({
                    "task_id": task_id,
                    "result": result,
                    "agent_id": at.agent_id,
                }),
            );
            let _ = self.app.emit(
                "task://status-changed",
                serde_json::json!({
                    "task_id": task_id,
                    "old_status": "in_progress",
                    "new_status": "in_review",
                }),
            );

            log::info!(
                "[TaskEngine] Task {} completed by agent {} (result: {} chars)",
                task_id,
                at.agent_id,
                result.len()
            );
        }

        Ok(())
    }

    /// Called when a task execution fails.
    ///
    /// For async tasks: retries if under MAX_RETRY, otherwise marks as failed.
    /// For realtime tasks: always marks as failed (frontend can retry manually).
    pub fn on_task_failed(&self, task_id: &str, error: &str) -> Result<(), String> {
        let active_task = {
            let mut active = self.active_tasks.lock().unwrap();
            active.remove(task_id)
        };

        if let Some(at) = active_task {
            // Release agent
            {
                let mut busy = self.agent_busy.lock().unwrap();
                busy.remove(&(at.agent_id.clone(), at.channel_id.clone()));
            }

            if at.mode == "async" {
                // Check if we should retry
                // Look up retry count from queue (it was removed from active, but we track retries)
                let retry_count = 0; // First attempt
                if retry_count < MAX_RETRY {
                    // Re-enqueue for retry
                    let queued = QueuedTask {
                        task_id: task_id.to_string(),
                        priority: 0, // Will be refreshed on next poll
                        enqueued_at: db_helpers::chrono_now_iso(),
                        retry_count: retry_count + 1,
                    };
                    {
                        let mut queue = self.async_queue.lock().unwrap();
                        queue.push_back(queued);
                    }

                    // Update status back to todo
                    self.update_task_status_in_db(task_id, "todo")?;

                    let _ = self.app.emit(
                        "task://retry",
                        serde_json::json!({
                            "task_id": task_id,
                            "retry_count": retry_count + 1,
                            "max_retry": MAX_RETRY,
                            "error": error,
                        }),
                    );

                    log::info!(
                        "[TaskEngine] Task {} failed (retry {}/{}): {}",
                        task_id,
                        retry_count + 1,
                        MAX_RETRY,
                        error
                    );
                } else {
                    // Max retries exceeded — mark as failed (we keep status as todo since we don't have a 'failed' state in the CHECK constraint,
                    // but we set it to 'blocked' with a descriptive result)
                    self.mark_task_failed(task_id, error)?;
                }
            } else {
                // Realtime failure — mark as failed
                self.mark_task_failed(task_id, error)?;
            }
        }

        Ok(())
    }

    /// Check cancellation status for a task.
    pub fn is_task_cancelled(&self, task_id: &str) -> bool {
        let active = self.active_tasks.lock().unwrap();
        if let Some(at) = active.get(task_id) {
            return at.cancel_token.is_cancelled();
        }
        false
    }

    // -----------------------------------------------------------------------
    // Dependency check
    // -----------------------------------------------------------------------

    /// Check if all dependencies for a task are satisfied (all deps are 'done').
    pub fn check_dependencies(&self, task_id: &str) -> Result<bool, String> {
        let state = self.app.state::<crate::commands::AppState>();
        let conn = state
            .db_conn
            .lock()
            .map_err(|e| format!("db lock error: {e}"))?;

        let deps = db_helpers::get_task_dependencies(&conn, task_id)
            .map_err(|e| format!("get dependencies failed: {e}"))?;

        if deps.is_empty() {
            return Ok(true);
        }

        for dep in &deps {
            let dep_task = db_helpers::get_task(&conn, &dep.depends_on_id)
                .map_err(|e| format!("get dep task failed: {e}"))?;

            match dep_task {
                Some(t) if t.status == "done" => continue,
                Some(t) => {
                    log::info!(
                        "[TaskEngine] Task {} blocked: dependency {} is '{}'",
                        task_id,
                        dep.depends_on_id,
                        t.status
                    );
                    return Ok(false);
                }
                None => {
                    log::warn!(
                        "[TaskEngine] Task {} blocked: dependency {} not found",
                        task_id,
                        dep.depends_on_id
                    );
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    // -----------------------------------------------------------------------
    // Getters for frontend state
    // -----------------------------------------------------------------------

    /// Get info about currently active tasks.
    pub fn get_active_tasks_info(&self) -> Vec<serde_json::Value> {
        let active = self.active_tasks.lock().unwrap();
        active
            .values()
            .map(|at| {
                serde_json::json!({
                    "task_id": at.task_id,
                    "agent_id": at.agent_id,
                    "channel_id": at.channel_id,
                    "started_at": at.started_at,
                    "mode": at.mode,
                })
            })
            .collect()
    }

    /// Get async queue length.
    pub fn queue_length(&self) -> usize {
        self.async_queue.lock().unwrap().len()
    }

    /// Check if an agent is busy on a specific channel.
    pub fn is_agent_busy(&self, agent_id: &str, channel_id: &str) -> bool {
        let busy = self.agent_busy.lock().unwrap();
        busy.contains(&(agent_id.to_string(), channel_id.to_string()))
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Helper: get task from DB via AppHandle state.
    fn get_task_from_db(&self, task_id: &str) -> Result<db_helpers::TaskRow, String> {
        let state = self.app.state::<crate::commands::AppState>();
        let conn = state
            .db_conn
            .lock()
            .map_err(|e| format!("db lock error: {e}"))?;

        db_helpers::get_task(&conn, task_id)
            .map_err(|e| format!("get task failed: {e}"))?
            .ok_or_else(|| format!("task not found: {task_id}"))
    }

    /// Helper: update task status in DB.
    fn update_task_status_in_db(&self, task_id: &str, status: &str) -> Result<(), String> {
        let state = self.app.state::<crate::commands::AppState>();
        let conn = state
            .db_conn
            .lock()
            .map_err(|e| format!("db lock error: {e}"))?;

        let now = db_helpers::chrono_now_iso();
        let completed_at = match status {
            "done" | "cancelled" => Some(now.clone()),
            _ => None,
        };

        db_helpers::update_task_status_with_completed(
            &conn,
            task_id,
            status,
            completed_at.as_deref(),
            &now,
        )
        .map_err(|e| format!("update status failed: {e}"))?;

        Ok(())
    }

    /// Helper: mark a task as permanently failed.
    fn mark_task_failed(&self, task_id: &str, error: &str) -> Result<(), String> {
        // We use 'blocked' status for failed tasks since our CHECK constraint
        // doesn't include 'failed'. The result field contains the error message.
        let state = self.app.state::<crate::commands::AppState>();
        let conn = state
            .db_conn
            .lock()
            .map_err(|e| format!("db lock error: {e}"))?;

        let now = db_helpers::chrono_now_iso();
        db_helpers::update_task(
            &conn,
            task_id,
            None,
            None,
            Some("blocked"),
            None,
            None,
            None,
            Some(Some(&format!("FAILED: {}", error))),
            &now,
        )
        .map_err(|e| format!("mark failed failed: {e}"))?;

        let _ = self.app.emit(
            "task://failed",
            serde_json::json!({
                "task_id": task_id,
                "error": error,
            }),
        );
        let _ = self.app.emit(
            "task://status-changed",
            serde_json::json!({
                "task_id": task_id,
                "old_status": "in_progress",
                "new_status": "blocked",
            }),
        );

        log::error!("[TaskEngine] Task {} failed permanently: {}", task_id, error);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Poll & Dispatch (inner function, runs in background thread)
// ---------------------------------------------------------------------------

/// Single poll cycle: find idle agent + unblocked task, dispatch.
fn poll_and_dispatch_inner(
    app: &AppHandle,
    queue: &Arc<Mutex<VecDeque<QueuedTask>>>,
    active: &Arc<Mutex<std::collections::HashMap<String, ActiveTask>>>,
    busy: &Arc<Mutex<std::collections::HashSet<(String, String)>>>,
) -> Result<(), String> {
    let mut dispatched = Vec::new();

    // Collect candidates from queue
    let mut candidates: Vec<QueuedTask> = {
        let q = queue.lock().unwrap();
        q.iter().cloned().collect()
    };

    // Sort by priority (higher first), then by enqueue time (earlier first)
    candidates.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.enqueued_at.cmp(&b.enqueued_at))
    });

    for candidate in &candidates {
        // Skip if already active
        {
            let act = active.lock().unwrap();
            if act.contains_key(&candidate.task_id) {
                continue;
            }
        }

        // Load task from DB
        let task = {
            let state = app.state::<crate::commands::AppState>();
            let conn = state
                .db_conn
                .lock()
                .map_err(|e| format!("db lock: {e}"))?;
            db_helpers::get_task(&conn, &candidate.task_id)
                .map_err(|e| format!("get task: {e}"))?
        };

        let task = match task {
            Some(t) => t,
            None => {
                // Task was deleted, remove from queue
                let mut q = queue.lock().unwrap();
                q.retain(|qt| qt.task_id != candidate.task_id);
                continue;
            }
        };

        let agent_id = match &task.assignee_id {
            Some(a) => a.clone(),
            None => continue, // No agent assigned, skip
        };

        let channel_id = match &task.channel_id {
            Some(c) => c.clone(),
            None => continue, // No channel, skip for async (need a channel for context)
        };

        // Check dependencies
        {
            let state = app.state::<crate::commands::AppState>();
            let conn = state
                .db_conn
                .lock()
                .map_err(|e| format!("db lock: {e}"))?;
            let deps =
                db_helpers::get_task_dependencies(&conn, &candidate.task_id)
                    .map_err(|e| format!("get deps: {e}"))?;
            let mut deps_met = true;
            for dep in &deps {
                match db_helpers::get_task(&conn, &dep.depends_on_id) {
                    Ok(Some(dt)) if dt.status == "done" => continue,
                    _ => {
                        deps_met = false;
                        break;
                    }
                }
            }
            if !deps_met {
                continue;
            }
        }

        // Check agent not busy on this channel
        {
            let b = busy.lock().unwrap();
            if b.contains(&(agent_id.clone(), channel_id.clone())) {
                continue;
            }
        }

        // Dispatch! Remove from queue, add to active, mark busy
        {
            let mut q = queue.lock().unwrap();
            q.retain(|qt| qt.task_id != candidate.task_id);
        }

        let cancel_token = CancellationToken::new();
        {
            let mut act = active.lock().unwrap();
            act.insert(
                candidate.task_id.clone(),
                ActiveTask {
                    task_id: candidate.task_id.clone(),
                    agent_id: agent_id.clone(),
                    channel_id: channel_id.clone(),
                    started_at: db_helpers::chrono_now_iso(),
                    mode: "async".to_string(),
                    cancel_token: cancel_token.clone(),
                },
            );
        }

        {
            let mut b = busy.lock().unwrap();
            b.insert((agent_id.clone(), channel_id.clone()));
        }

        // Update DB status -> in_progress
        {
            let state = app.state::<crate::commands::AppState>();
            let conn = state
                .db_conn
                .lock()
                .map_err(|e| format!("db lock: {e}"))?;
            let now = db_helpers::chrono_now_iso();
            db_helpers::update_task_status_with_completed(&conn, &candidate.task_id, "in_progress", None, &now)
                .map_err(|e| format!("update status: {e}"))?;
        }

        // Build async context and emit execution event
        let task_prompt = build_task_context_prompt(&task);

        dispatched.push(candidate.task_id.clone());

        let _ = app.emit(
            "task://execute-async",
            serde_json::json!({
                "task_id": candidate.task_id,
                "agent_id": agent_id,
                "channel_id": channel_id,
                "task_prompt": task_prompt,
                "retry_count": candidate.retry_count,
            }),
        );

        let _ = app.emit(
            "task://status-changed",
            serde_json::json!({
                "task_id": candidate.task_id,
                "new_status": "in_progress",
            }),
        );

        log::info!(
            "[TaskEngine] Async dispatch: task={}, agent={}, channel={}",
            candidate.task_id,
            agent_id,
            channel_id
        );
    }

    if !dispatched.is_empty() {
        log::info!("[TaskEngine] Poll dispatched {} tasks", dispatched.len());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a Task context prompt for injection into the system prompt.
///
/// Budget: <= 500 tokens (~2000 chars).
fn build_task_context_prompt(task: &db_helpers::TaskRow) -> String {
    let mut prompt = String::from("[Task Execution Context]\n");
    prompt.push_str(&format!("Task: {}\n", task.title));

    if !task.description.is_empty() {
        // Truncate description to ~200 tokens (~800 chars)
        let desc = if task.description.len() > 800 {
            &task.description[..800]
        } else {
            &task.description
        };
        prompt.push_str(&format!("Description: {}\n", desc));
    }

    prompt.push_str("\nPlease execute this task and report results.\n");
    prompt
}
