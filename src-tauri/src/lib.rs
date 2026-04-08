pub mod commands;
pub mod context;
pub mod runtime;
pub mod storage;

use runtime::registry::RuntimeRegistry;
use std::sync::Mutex;

/// Application state shared across all Tauri commands.
pub struct AppState {
    /// Agent runtime registry for managing agent runtimes (Claude Code, etc.)
    pub agent_runtime_registry: Mutex<RuntimeRegistry>,
    /// Current agent session state (session_id + process tracking)
    pub agent_session: Mutex<AgentSessionState>,
}

/// State tracking for an active agent session.
#[derive(Default)]
pub struct AgentSessionState {
    /// The current session ID (if any)
    pub session_id: Option<String>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let registry = runtime::registry::create_default_registry();

    tauri::Builder::default()
        .manage(AppState {
            agent_runtime_registry: Mutex::new(registry),
            agent_session: Mutex::new(AgentSessionState::default()),
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            runtime::commands::scan_agent_runtimes,
            runtime::commands::list_agent_runtimes,
            runtime::commands::runtime_execute,
            runtime::commands::runtime_session_start,
            runtime::commands::runtime_session_stop,
            storage::keyring::store_api_key,
            storage::keyring::has_api_key,
            storage::keyring::delete_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
