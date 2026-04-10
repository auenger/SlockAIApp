pub mod commands;
pub mod context;
pub mod runtime;
pub mod storage;
pub mod workspace;

use commands::AppState;
// RuntimeRegistry is used at runtime; import kept for future use.
use std::sync::Mutex;
use tauri::Manager;
use workspace::manager::AgentManager;

/// Default workspace directory name.
const DEFAULT_WORKSPACE_DIR: &str = "workspaces";

/// Resolve the workspace root path.
///
/// Uses the app's data directory (managed by Tauri) to store workspace data.
fn resolve_workspace_root(app: &tauri::AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(DEFAULT_WORKSPACE_DIR)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let registry = runtime::registry::create_default_registry();

    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize workspace root from app data dir
            let workspace_root = resolve_workspace_root(app.handle());
            let agent_manager = AgentManager::new(&workspace_root);

            // Initialize SQLite database
            let db_conn = storage::db::init_database(&workspace_root)
                .expect("Failed to initialize SQLite database");

            log::info!(
                "[App] SQLite database initialized at {}",
                workspace_root.join("agentszone.db").display()
            );

            // Migrate existing JSON data into SQLite (idempotent, only runs once)
            if let Err(e) = storage::db::migrate_from_files(&db_conn, &workspace_root) {
                log::warn!("[App] Data migration from files failed: {}", e);
            }

            app.manage(AppState {
                agent_runtime_registry: Mutex::new(registry),
                agent_session: Mutex::new(commands::AgentSessionState::default()),
                agent_manager: Mutex::new(agent_manager),
                db_conn: Mutex::new(db_conn),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            runtime::commands::scan_agent_runtimes,
            runtime::commands::list_agent_runtimes,
            runtime::commands::get_runtime_info,
            runtime::commands::runtime_execute,
            runtime::commands::runtime_session_start,
            runtime::commands::runtime_session_stop,
            storage::keyring::store_api_key,
            storage::keyring::has_api_key,
            storage::keyring::delete_api_key,
            storage::keyring::list_api_keys,
            storage::keyring::verify_api_key,
            commands::init_workspace,
            commands::get_workspace_status,
            commands::create_agent,
            commands::list_agents,
            commands::switch_agent,
            commands::get_active_agent,
            commands::delete_agent,
            commands::update_agent,
            commands::get_agent_identity,
            commands::get_agent_context,
            commands::get_agent_runtime_status,
            commands::list_workspace_dir,
            commands::read_workspace_file,
            commands::thread::create_thread,
            commands::thread::list_threads,
            commands::thread::get_thread,
            commands::thread::delete_thread,
            commands::thread::send_message,
            commands::thread::save_agent_response,
            commands::thread::load_thread_messages,
            commands::channel::create_channel,
            commands::channel::list_channels,
            commands::channel::get_channel,
            commands::channel::update_channel,
            commands::channel::delete_channel,
            commands::channel::add_channel_member,
            commands::channel::remove_channel_member,
            commands::channel::send_channel_message,
            commands::channel::save_channel_response,
            commands::list_skills,
            commands::add_skill,
            commands::update_skill,
            commands::delete_skill,
            commands::get_skill_status,
            commands::activity::log_activity,
            commands::activity::list_activities,
            commands::activity::clear_activities,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
