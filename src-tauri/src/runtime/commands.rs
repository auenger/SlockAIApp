//! Tauri IPC commands for the agent runtime system.
//!
//! Provides invoke handlers for scanning, listing, and executing
//! agent runtimes, as well as session management.

use crate::AppState;
use super::ExecuteParams;
use tauri::{AppHandle, Emitter};

/// Scan all registered agent runtimes and detect their availability.
/// Returns a list of runtime info objects with detection results.
#[tauri::command]
pub fn scan_agent_runtimes(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<super::AgentRuntimeInfo>, String> {
    let mut registry = state
        .agent_runtime_registry
        .lock()
        .map_err(|e| e.to_string())?;
    Ok(registry.scan_all())
}

/// List all registered agent runtimes using cached detection data.
/// Faster than scan since it does not re-run detection.
#[tauri::command]
pub fn list_agent_runtimes(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<super::AgentRuntimeInfo>, String> {
    let registry = state
        .agent_runtime_registry
        .lock()
        .map_err(|e| e.to_string())?;
    Ok(registry.list_all())
}

/// Execute a message on a specific agent runtime.
/// Spawns the CLI process and emits `agent://chunk` events to the frontend
/// as StreamEvents arrive.
#[tauri::command]
pub async fn runtime_execute(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    runtime_id: String,
    message: String,
    session_id: Option<String>,
    system_prompt: Option<String>,
) -> Result<(), String> {
    // Build execution parameters
    let params = ExecuteParams {
        message,
        session_id: session_id.clone(),
        workspace: None,
        system_prompt,
        timeout_secs: 120,
    };

    log::info!(
        "[runtime_execute] runtime_id={}, has_session_id={}",
        runtime_id,
        session_id.is_some()
    );

    // Get the receiver from the runtime while briefly holding the lock
    let receiver = {
        let registry = state
            .agent_runtime_registry
            .lock()
            .map_err(|e| e.to_string())?;
        let runtime = registry.get_runtime_instance(&runtime_id)?;
        runtime.execute(params)?
    };

    // Spawn thread to forward events to frontend via Tauri events
    std::thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            let _ = app.emit("agent://chunk", &event);
            if event.is_done {
                break;
            }
        }
    });

    Ok(())
}

/// Start a new agent runtime session.
/// Verifies the runtime is ready and generates a new session ID (UUID).
#[tauri::command]
pub async fn runtime_session_start(
    state: tauri::State<'_, AppState>,
    runtime_id: String,
) -> Result<String, String> {
    // Verify the runtime exists and is ready
    {
        let registry = state
            .agent_runtime_registry
            .lock()
            .map_err(|e| e.to_string())?;
        let runtime = registry.get_runtime_instance(&runtime_id)?;
        if !runtime.is_ready() {
            return Err(format!(
                "Runtime '{}' is not ready (CLI not found or unhealthy)",
                runtime_id
            ));
        }
    }

    // Generate a new session ID using a simple approach
    let session_id = format!(
        "{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );

    // Store the session ID
    {
        let mut session = state.agent_session.lock().map_err(|e| e.to_string())?;
        session.session_id = Some(session_id.clone());
    }

    log::info!("[runtime_session_start] session_id={}", session_id);
    Ok(session_id)
}

/// Stop the current agent runtime session.
/// Clears the session state.
#[tauri::command]
pub fn runtime_session_stop(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut session = state.agent_session.lock().map_err(|e| e.to_string())?;
    session.session_id = None;
    log::info!("[runtime_session_stop] session cleared");
    Ok(())
}
