//! Tauri IPC commands for Thread (1-on-1 chat) operations.
//!
//! Provides CRUD operations for Threads, plus the `send_message` command
//! that integrates with the Claude Code Runtime for streaming responses.
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

use crate::storage::jsonl::JsonlStore;
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
    let session_id = thread::generate_id();
    let now = thread::now_iso();

    let new_thread = Thread {
        id: thread_id,
        agent_id: agent_id.clone(),
        title: format!("Thread with {}", agent.identity.name),
        session_id: Some(session_id),
        messages: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
    };

    let conv_dir = workspace.conversations_dir();
    let store = ThreadStore::new(&conv_dir);
    store.save(&new_thread).map_err(|e| format!("save failed: {e}"))?;

    log::info!(
        "[create_thread] thread_id={}, agent_id={}",
        new_thread.id,
        new_thread.agent_id
    );

    Ok(new_thread)
}

/// List all threads for a specific agent.
#[tauri::command]
pub fn list_threads(
    state: tauri::State<'_, AppState>,
    agent_id: String,
) -> Result<Vec<ThreadInfo>, String> {
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

    log::info!("[delete_thread] thread_id={}, agent_id={}", thread_id, agent_id);
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

        // Add user message
        let user_msg = ThreadMessage {
            id: thread::generate_id(),
            role: "user".to_string(),
            content: message.clone(),
            timestamp: thread::now_iso(),
        };
        thread.messages.push(user_msg.clone());
        thread.updated_at = thread::now_iso();

        // Save updated thread (JSON)
        store.save(&thread).map_err(|e| format!("save failed: {e}"))?;

        // Also persist to JSONL (append-only)
        let jsonl = jsonl_store_for_agent(&manager, &agent_id)?;
        if let Err(e) = jsonl.append_message(&thread_id, &user_msg) {
            log::warn!("[send_message] JSONL append failed for thread {}: {}", thread_id, e);
        }

        // Get workspace path for runtime execution
        let workspace_path = workspace.base_path().to_string_lossy().to_string();
        let session_id = thread.session_id.clone();

        // Get runtime for execution
        let receiver = {
            let registry = state
                .agent_runtime_registry
                .lock()
                .map_err(|e| e.to_string())?;
            let runtime = registry.get_runtime_instance("claude-code")?;

            let params = ExecuteParams {
                message,
                session_id,
                workspace: Some(workspace_path),
                system_prompt: None,
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

            while let Ok(event) = receiver.recv() {
                // Capture session_id from result
                if event.is_done {
                    result_session_id = event.session_id.clone();
                }
                // Accumulate assistant text
                if event.msg_type.as_deref() == Some("assistant") && !event.text.is_empty() {
                    full_response.push_str(&event.text);
                }
                // Forward event to frontend
                let _ = app_clone.emit("agent://chunk", &event);
                if event.is_done {
                    break;
                }
            }

            // Emit a thread-response event so the frontend can save the agent response
            if !full_response.is_empty() {
                let _ = app_clone.emit("agent://thread-response", serde_json::json!({
                    "thread_id": thread_id_clone,
                    "agent_id": agent_id_clone,
                    "content": full_response,
                    "session_id": result_session_id,
                }));
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

    store.save(&thread).map_err(|e| format!("save failed: {e}"))?;

    // Also persist to JSONL (append-only)
    let jsonl = jsonl_store_for_agent(&manager, &agent_id)?;
    if let Err(e) = jsonl.append_message(&thread_id, &agent_msg) {
        log::warn!("[save_agent_response] JSONL append failed for thread {}: {}", thread_id, e);
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
