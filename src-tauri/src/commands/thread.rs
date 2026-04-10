//! Tauri IPC commands for Thread (1-on-1 chat) operations.
//!
//! Provides CRUD operations for Threads, plus the `send_message` command
//! that integrates with the Agent Runtime system for streaming responses.
//! Runtime routing is based on each agent's `runtime_type` field --
//! the system automatically uses the correct backend (Claude Code, Codex, etc.).
//!
//! ## Persistence Strategy
//!
//! Messages are persisted in **two** complementary formats:
//!
//! 1. **Thread JSON** (`thread_{id}.json`) -- Full thread object including all
//!    messages, metadata, and session info. Used as the primary data source.
//!
//! 2. **JSONL** (`{thread_id}.jsonl`) -- Append-only log of individual messages.
//!    Each message is appended as a single JSON line. Used for crash recovery
//!    and as a redundant backup. On thread load, JSONL messages are reconciled
//!    with the thread JSON to recover any lost data.
//!
//! 3. **SQLite** -- Thread metadata (id, agent_id, title, message_count, jsonl_path)
//!    is tracked in the `threads` table. This enables fast listing without scanning
//!    JSON files, and keeps metadata in sync with the file-based storage.

use crate::storage::jsonl::JsonlStore;
use crate::storage::activity::{ActivityStore, ActivityType, create_entry};
use crate::storage::db_helpers;
use crate::AppState;
use crate::runtime::ExecuteParams;
use crate::workspace::thread::{self, Thread, ThreadInfo, ThreadMessage, ThreadStore};
use tauri::{AppHandle, Emitter};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a JsonlStore for the given agent's thread JSONL directory.
///
/// The JSONL files are stored in `conversations/threads/` under the agent workspace.
fn jsonl_store_for_agent(
    manager: &crate::workspace::manager::AgentManager,
    agent_id: &str,
) -> Result<JsonlStore, String> {
    let workspace = manager
        .get_workspace(agent_id)
        .ok_or_else(|| format!("workspace not found for agent: {agent_id}"))?;
    let threads_dir = workspace.conversations_dir().join("threads");
    Ok(JsonlStore::new(threads_dir))
}

// ---------------------------------------------------------------------------
// Thread CRUD
// ---------------------------------------------------------------------------

/// Create a new Thread for a specific Agent.
#[tauri::command]
pub fn create_thread(
    state: tauri::State<'_, AppState>,
    agent_id: String,
) -> Result<Thread, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    // Verify the agent exists
    let agent = manager
        .get_agent(&agent_id)
        .ok_or_else(|| format!("agent not found: {agent_id}"))?;

    let workspace = manager.get_workspace(&agent_id)
        .ok_or_else(|| format!("workspace not found for agent: {agent_id}"))?;

    let thread_id = thread::generate_id();
    let now = thread::now_iso();

    let new_thread = Thread {
        id: thread_id.clone(),
        agent_id: agent_id.clone(),
        title: format!("Thread with {}", agent.identity.name),
        session_id: None,
        messages: Vec::new(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    let conv_dir = workspace.conversations_dir();
    let store = ThreadStore::new(&conv_dir);
    store.save(&new_thread).map_err(|e| format!("save failed: {e}"))?;

    // Insert thread metadata into SQLite
    {
        let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
        let jsonl_rel_path = format!("agents/{}/conversations/threads/{}.jsonl", agent_id, new_thread.id);
        let thread_row = db_helpers::ThreadRow {
            id: new_thread.id.clone(),
            agent_id: agent_id.clone(),
            title: new_thread.title.clone(),
            session_id: new_thread.session_id.clone(),
            message_count: 0,
            jsonl_path: Some(jsonl_rel_path),
            created_at: new_thread.created_at.clone(),
            updated_at: new_thread.updated_at.clone(),
        };
        if let Err(e) = db_helpers::insert_thread(&db_conn, &thread_row) {
            log::warn!("[create_thread] Failed to insert thread into SQLite: {}", e);
        }
    }

    log::info!(
        "[create_thread] thread_id={}, agent_id={}",
        new_thread.id,
        new_thread.agent_id
    );

    // Log activity (dual-write: JSONL + SQLite)
    {
        let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
        let activity_store = ActivityStore::new(manager.workspace_root());
        let entry = create_entry(
            ActivityType::ConversationStarted,
            Some(agent_id.clone()),
            format!("Conversation started with {}", agent.identity.name),
            serde_json::json!({ "thread_id": new_thread.id }),
        );
        if let Err(e) = activity_store.append(&entry) {
            log::warn!("[create_thread] Failed to log activity to JSONL: {}", e);
        }

        // Dual-write activity to SQLite
        let activity_type_str = serde_json::to_string(&ActivityType::ConversationStarted)
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
            log::warn!("[create_thread] Failed to log activity to SQLite: {}", e);
        }
    }

    Ok(new_thread)
}

/// List all threads for a specific agent.
#[tauri::command]
pub fn list_threads(
    state: tauri::State<'_, AppState>,
    agent_id: String,
) -> Result<Vec<ThreadInfo>, String> {
    // Try SQLite first for fast listing
    {
        let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
        let db_threads = db_helpers::list_threads_by_agent(&db_conn, &agent_id);
        if let Ok(rows) = db_threads {
            if !rows.is_empty() {
                // We have SQLite data -- use it for fast listing
                let thread_infos: Vec<ThreadInfo> = rows.into_iter().map(|r| {
                    ThreadInfo {
                        id: r.id,
                        agent_id: r.agent_id,
                        title: r.title,
                        preview: String::new(), // preview not stored in SQLite metadata
                        message_count: r.message_count as usize,
                        created_at: r.created_at,
                        updated_at: r.updated_at,
                    }
                }).collect();
                return Ok(thread_infos);
            }
        }
    }

    // Fallback to file-based listing if SQLite has no data
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let workspace = manager.get_workspace(&agent_id)
        .ok_or_else(|| format!("workspace not found for agent: {agent_id}"))?;

    let conv_dir = workspace.conversations_dir();
    let store = ThreadStore::new(&conv_dir);
    store.list().map_err(|e| format!("list failed: {e}"))
}

/// Get a single thread by ID.
///
/// Loads the thread JSON and reconciles with any JSONL messages that may
/// not have been saved to the JSON (e.g. after a crash during write).
#[tauri::command]
pub fn get_thread(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    thread_id: String,
) -> Result<Thread, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let workspace = manager.get_workspace(&agent_id)
        .ok_or_else(|| format!("workspace not found for agent: {agent_id}"))?;

    let conv_dir = workspace.conversations_dir();
    let store = ThreadStore::new(&conv_dir);
    let mut thread = store.load(&thread_id).map_err(|e| format!("load failed: {e}"))?;

    // Reconcile with JSONL: recover any messages not in the JSON
    let jsonl = jsonl_store_for_agent(&manager, &agent_id)?;
    let jsonl_messages = jsonl.load_messages(&thread_id).unwrap_or_default();

    if jsonl_messages.len() > thread.messages.len() {
        log::info!(
            "[get_thread] Recovering {} messages from JSONL for thread {}",
            jsonl_messages.len() - thread.messages.len(),
            thread_id
        );
        thread.messages = jsonl_messages;
        thread.updated_at = thread::now_iso();
        // Persist the reconciled state back to JSON
        if let Err(e) = store.save(&thread) {
            log::warn!("[get_thread] Failed to save reconciled thread: {}", e);
        }
    }

    Ok(thread)
}

/// Delete a thread by ID.
#[tauri::command]
pub fn delete_thread(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    thread_id: String,
) -> Result<(), String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let workspace = manager.get_workspace(&agent_id)
        .ok_or_else(|| format!("workspace not found for agent: {agent_id}"))?;

    let conv_dir = workspace.conversations_dir();
    let store = ThreadStore::new(&conv_dir);
    store.delete(&thread_id).map_err(|e| format!("delete failed: {e}"))?;

    // Also delete JSONL file
    let jsonl = jsonl_store_for_agent(&manager, &agent_id)?;
    if let Err(e) = jsonl.delete_thread(&thread_id) {
        log::warn!("[delete_thread] JSONL delete failed for thread {}: {}", thread_id, e);
    }

    // Delete from SQLite
    {
        let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
        if let Err(e) = db_helpers::delete_thread(&db_conn, &thread_id) {
            log::warn!("[delete_thread] SQLite delete failed for thread {}: {}", thread_id, e);
        }
    }

    log::info!("[delete_thread] thread_id={}, agent_id={}", thread_id, agent_id);

    // Log activity (dual-write: JSONL + SQLite)
    {
        let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
        let activity_store = ActivityStore::new(manager.workspace_root());
        let entry = create_entry(
            ActivityType::ConversationEnded,
            Some(agent_id.clone()),
            format!("Conversation {} ended", thread_id),
            serde_json::json!({ "thread_id": thread_id }),
        );
        if let Err(e) = activity_store.append(&entry) {
            log::warn!("[delete_thread] Failed to log activity to JSONL: {}", e);
        }

        let activity_type_str = serde_json::to_string(&ActivityType::ConversationEnded)
            .unwrap_or_else(|_| "\"system\"".to_string())
            .trim_matches('"')
            .to_string();
        let db_row = db_helpers::ActivityLogRow {
            id: entry.id,
            timestamp: entry.timestamp,
            activity_type: activity_type_str,
            agent_id: entry.agent_id,
            workspace_id: entry.workspace_id,
            summary: entry.summary,
            details_json: serde_json::to_string(&entry.details).unwrap_or_else(|_| "{}".to_string()),
        };
        if let Err(e) = db_helpers::insert_activity(&db_conn, &db_row) {
            log::warn!("[delete_thread] Failed to log activity to SQLite: {}", e);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Send message (with Runtime integration)
// ---------------------------------------------------------------------------

/// Send a message in a thread and trigger Claude Code Runtime execution.
///
/// 1. Saves the user message to the thread
/// 2. Calls `runtime_execute` to start streaming
/// 3. The runtime will emit `agent://chunk` events to the frontend
#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    agent_id: String,
    thread_id: String,
    message: String,
) -> Result<Thread, String> {
    // 1. Load thread and add user message
    let thread = {
        let manager = state
            .agent_manager
            .lock()
            .map_err(|e| format!("lock error: {e}"))?;

        let workspace = manager.get_workspace(&agent_id)
            .ok_or_else(|| format!("workspace not found for agent: {agent_id}"))?;

        let conv_dir = workspace.conversations_dir();
        let store = ThreadStore::new(&conv_dir);
        let mut thread = store.load(&thread_id).map_err(|e| format!("load failed: {e}"))?;

        // Track whether this is the first message BEFORE adding user message.
        // First message: do NOT pass --resume to Claude CLI (no valid session yet).
        // Subsequent messages: use the Claude CLI session_id returned from the first response.
        let is_first_message = thread.messages.is_empty();

        // Add user message
        let user_msg = ThreadMessage {
            id: thread::generate_id(),
            role: "user".to_string(),
            content: message.clone(),
            timestamp: thread::now_iso(),
        };
        thread.messages.push(user_msg.clone());
        thread.updated_at = thread::now_iso();

        let message_count = thread.messages.len() as i64;

        // Save updated thread (JSON)
        store.save(&thread).map_err(|e| format!("save failed: {e}"))?;

        // Also persist to JSONL (append-only)
        let jsonl = jsonl_store_for_agent(&manager, &agent_id)?;
        if let Err(e) = jsonl.append_message(&thread_id, &user_msg) {
            log::warn!("[send_message] JSONL append failed for thread {}: {}", thread_id, e);
        }

        // Update thread metadata in SQLite
        {
            let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
            if let Err(e) = db_helpers::update_thread_meta(
                &db_conn,
                &thread_id,
                message_count,
                &thread.updated_at,
            ) {
                log::warn!("[send_message] Failed to update thread meta in SQLite: {}", e);
            }
        }

        // Build context prefix using ContextBuilder (same as Channel mode)
        let workspace_root = workspace.base_path();
        let builder = crate::context::ContextBuilder::new(workspace_root);
        let context_prefix = builder
            .build_context_prefix(&agent_id)
            .unwrap_or_default();

        // Get workspace path for runtime execution
        let workspace_path = workspace.base_path().to_string_lossy().to_string();

        // Session ID logic:
        // - First message: None (Claude CLI will create a new session)
        // - Subsequent messages: use the session_id returned by Claude CLI
        //   from the first response (stored via save_agent_response)
        let session_id = if is_first_message {
            log::info!(
                "[send_message] First message in thread {}, starting new Claude session",
                thread_id
            );
            None
        } else {
            log::info!(
                "[send_message] Resuming session {} for thread {}",
                thread.session_id.as_deref().unwrap_or("None"),
                thread_id
            );
            thread.session_id.clone()
        };

        // Resolve the agent's runtime_type for routing
        let runtime_id = manager.get_agent(&agent_id)
            .ok_or_else(|| format!("agent not found: {agent_id}"))?
            .identity.runtime_type.runtime_id().to_string();

        // Get runtime for execution -- route based on agent's runtime_type
        let receiver = {
            let registry = state
                .agent_runtime_registry
                .lock()
                .map_err(|e| e.to_string())?;

            let runtime = registry.get_runtime_instance(&runtime_id)?;

            // Health check: verify the runtime is available before executing
            if !runtime.is_ready() {
                let info = registry.get_runtime(&runtime_id);
                let install_hint = info
                    .map(|i| i.install_hint.clone())
                    .unwrap_or_default();
                let error_msg = format!(
                    "{} runtime is not available. Please install: {}",
                    runtime.name(),
                    install_hint,
                );
                // Emit runtime unavailable event for frontend UX
                let _ = app.emit("runtime://unavailable", serde_json::json!({
                    "agent_id": agent_id,
                    "thread_id": thread_id,
                    "runtime_id": runtime_id,
                    "runtime_name": runtime.name(),
                    "install_hint": install_hint,
                    "error": error_msg,
                }));
                return Err(error_msg);
            }

            let params = ExecuteParams {
                message,
                session_id,
                workspace: Some(workspace_path),
                system_prompt: Some(context_prefix),
                timeout_secs: 120,
            };

            runtime.execute(params)?
        };

        // Spawn thread to forward streaming events to frontend
        let app_clone = app.clone();
        let thread_id_clone = thread_id.clone();
        let agent_id_clone = agent_id.clone();
        std::thread::spawn(move || {
            let mut full_response = String::new();
            let mut result_session_id: Option<String> = None;
            let mut event_count = 0u32;

            log::info!(
                "[forward_thread] Starting event forwarding for thread {}",
                thread_id_clone
            );

            while let Ok(event) = receiver.recv() {
                event_count += 1;

                // Capture session_id from result
                if event.is_done {
                    result_session_id = event.session_id.clone();
                    log::info!(
                        "[forward_thread] Stream done, session_id={:?}, total_events={}",
                        result_session_id, event_count
                    );
                }
                // Accumulate assistant text
                if event.msg_type.as_deref() == Some("assistant") && !event.text.is_empty() {
                    full_response.push_str(&event.text);
                }
                // Forward event to frontend
                if let Err(e) = app_clone.emit("agent://chunk", &event) {
                    log::error!("[forward_thread] Failed to emit chunk event: {}", e);
                }
                if event.is_done {
                    break;
                }
            }

            log::info!(
                "[forward_thread] Finished. response_len={}, events={}",
                full_response.len(),
                event_count
            );

            // Emit a thread-response event so the frontend can save the agent response
            if !full_response.is_empty() {
                log::info!(
                    "[forward_thread] Emitting thread-response, session_id={:?}",
                    result_session_id
                );
                if let Err(e) = app_clone.emit("agent://thread-response", serde_json::json!({
                    "thread_id": thread_id_clone,
                    "agent_id": agent_id_clone,
                    "content": full_response,
                    "session_id": result_session_id,
                })) {
                    log::error!("[forward_thread] Failed to emit thread-response: {}", e);
                }
            } else {
                log::warn!(
                    "[forward_thread] No assistant text accumulated after {} events",
                    event_count
                );
            }
        });

        thread
    };

    Ok(thread)
}

/// Save an agent response to a thread (called after streaming completes).
#[tauri::command]
pub fn save_agent_response(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    thread_id: String,
    content: String,
    session_id: Option<String>,
) -> Result<Thread, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let workspace = manager.get_workspace(&agent_id)
        .ok_or_else(|| format!("workspace not found for agent: {agent_id}"))?;

    let conv_dir = workspace.conversations_dir();
    let store = ThreadStore::new(&conv_dir);
    let mut thread = store.load(&thread_id).map_err(|e| format!("load failed: {e}"))?;

    // Update session_id if provided
    if let Some(sid) = session_id {
        thread.session_id = Some(sid);
    }

    // Add agent message
    let agent_msg = ThreadMessage {
        id: thread::generate_id(),
        role: "agent".to_string(),
        content,
        timestamp: thread::now_iso(),
    };
    thread.messages.push(agent_msg.clone());
    thread.updated_at = thread::now_iso();

    let message_count = thread.messages.len() as i64;

    store.save(&thread).map_err(|e| format!("save failed: {e}"))?;

    // Also persist to JSONL (append-only)
    let jsonl = jsonl_store_for_agent(&manager, &agent_id)?;
    if let Err(e) = jsonl.append_message(&thread_id, &agent_msg) {
        log::warn!("[save_agent_response] JSONL append failed for thread {}: {}", thread_id, e);
    }

    // Update thread metadata in SQLite
    {
        let db_conn = state.db_conn.lock().map_err(|e| format!("lock error: {e}"))?;
        if let Err(e) = db_helpers::update_thread_meta(
            &db_conn,
            &thread_id,
            message_count,
            &thread.updated_at,
        ) {
            log::warn!("[save_agent_response] Failed to update thread meta in SQLite: {}", e);
        }
    }

    Ok(thread)
}

/// Load messages for a thread from JSONL storage.
///
/// This is primarily for crash recovery or when the JSON file is missing
/// but the JSONL log still exists. Returns messages in chronological order.
#[tauri::command]
pub fn load_thread_messages(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    thread_id: String,
    limit: Option<usize>,
) -> Result<Vec<ThreadMessage>, String> {
    let manager = state
        .agent_manager
        .lock()
        .map_err(|e| format!("lock error: {e}"))?;

    let jsonl = jsonl_store_for_agent(&manager, &agent_id)?;

    match limit {
        Some(n) => jsonl.load_recent_messages(&thread_id, n),
        None => jsonl.load_messages(&thread_id),
    }
    .map_err(|e| format!("load failed: {e}"))
}
