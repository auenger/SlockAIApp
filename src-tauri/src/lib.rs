pub mod bridge;
pub mod cli;
pub mod context;
pub mod runtime;
pub mod workspace;

#[cfg(feature = "tauri-app")]
pub mod commands;
#[cfg(feature = "tauri-app")]
pub mod storage;
#[cfg(feature = "tauri-app")]
pub mod task_engine;

#[cfg(feature = "tauri-app")]
use commands::AppState;
#[cfg(feature = "tauri-app")]
use std::sync::Mutex;
#[cfg(feature = "tauri-app")]
use tauri::Manager;
#[cfg(feature = "tauri-app")]
use workspace::manager::AgentManager;

/// Resolve the workspace root path.
///
/// Uses `~/.agentszone/` as the unified data directory across all platforms.
pub fn resolve_workspace_root() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".agentszone")
}

#[cfg(feature = "tauri-app")]
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

            // Initialize workspace root from ~/.agentszone/
            let workspace_root = resolve_workspace_root();
            let mut agent_manager = AgentManager::new(&workspace_root);

            // Ensure workspace directory structure exists
            if let Err(e) = agent_manager.initialize_workspace() {
                log::error!("[App] CRITICAL: Workspace initialization failed: {}", e);
                log::info!("[App] Attempting workspace self-heal...");
                // Attempt self-heal: re-create the workspace from scratch
                if let Err(heal_err) = agent_manager.initialize_workspace() {
                    log::error!("[App] Workspace self-heal also failed: {}", heal_err);
                } else {
                    log::info!("[App] Workspace self-heal succeeded");
                }
            }

            // Load existing agents from disk into memory
            if let Err(e) = agent_manager.load() {
                log::error!("[App] CRITICAL: Failed to load agents from disk: {}", e);
                log::info!("[App] Attempting workspace re-initialization...");
                // Re-initialize and try loading again
                if let Err(heal_err) = agent_manager.initialize_workspace() {
                    log::error!("[App] Re-initialization failed: {}", heal_err);
                }
                if let Err(reload_err) = agent_manager.load() {
                    log::error!("[App] Re-load after re-init also failed: {}", reload_err);
                }
            }

            log::info!(
                "[App] Loaded {} agents from workspace",
                agent_manager.list_agents().len()
            );

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
                a2a_server: Mutex::new(commands::a2a_server::A2AServerState::new()),
            });

            // Initialize TaskEngine and start background poll thread
            let task_engine = task_engine::TaskEngine::new(app.handle().clone());
            task_engine.start_poll_thread();
            app.manage(task_engine);

            // Initialize CollaborationState for A2A multi-agent features
            let artifacts_dir = workspace_root.join("artifacts");
            let collaboration_state = commands::collaboration::CollaborationState::new(&artifacts_dir);
            app.manage(collaboration_state);

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
            commands::health_check_workspace,
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
            commands::open_in_finder,
            commands::thread::create_thread,
            commands::thread::list_threads,
            commands::thread::get_thread,
            commands::thread::delete_thread,
            commands::thread::send_message,
            commands::thread::save_agent_response,
            commands::thread::load_thread_messages,
            commands::thread::list_all_threads,
            commands::thread::rename_thread,
            commands::channel::create_channel,
            commands::channel::list_channels,
            commands::channel::get_channel,
            commands::channel::update_channel,
            commands::channel::delete_channel,
            commands::channel::add_channel_member,
            commands::channel::remove_channel_member,
            commands::channel::send_channel_message,
            commands::channel::save_channel_response,
            commands::channel::compact_channel,
            commands::list_skills,
            commands::add_skill,
            commands::update_skill,
            commands::delete_skill,
            commands::get_skill_status,
            commands::activity::log_activity,
            commands::activity::list_activities,
            commands::activity::clear_activities,
            commands::task::create_task,
            commands::task::list_tasks,
            commands::task::get_task,
            commands::task::update_task,
            commands::task::delete_task,
            commands::task::update_task_status,
            commands::task::assign_task,
            commands::task::cancel_task,
            commands::task::add_task_dependency,
            commands::task::remove_task_dependency,
            commands::task::get_task_history,
            commands::task::get_task_dependencies,
            commands::task::get_dependent_tasks,
            commands::task::get_child_tasks,
            commands::task::execute_task,
            commands::task::cancel_task_execution,
            commands::task::report_task_completed,
            commands::task::report_task_failed,
            commands::task::get_task_engine_status,
            commands::task_suggestion::confirm_task_suggestions,
            commands::task_suggestion::dismiss_task_suggestions,
            commands::remote_connection::remote_connection_create,
            commands::remote_connection::remote_connection_list,
            commands::remote_connection::remote_connection_update,
            commands::remote_connection::remote_connection_delete,
            commands::remote_connection::remote_connection_test,
            commands::remote_connection::remote_connection_health_all,
            commands::remote_connection::remote_connection_get_agent_card,
            commands::collaboration::collaboration_delegate,
            commands::collaboration::collaboration_list_delegations,
            commands::collaboration::collaboration_cancel_delegation,
            commands::collaboration::collaboration_retry_delegation,
            commands::collaboration::collaboration_list_artifacts,
            commands::collaboration::collaboration_get_artifact,
            commands::collaboration::collaboration_search_artifacts,
            commands::collaboration::collaboration_register_artifact,
            commands::collaboration::collaboration_register_push_url,
            commands::collaboration::collaboration_list_push_configs,
            commands::collaboration::collaboration_unregister_push_url,
            commands::collaboration::collaboration_process_push_event,
            commands::a2a_server::start_a2a_server,
            commands::a2a_server::stop_a2a_server,
            commands::a2a_server::get_a2a_server_status,
            commands::a2a_server::get_local_ip_addresses,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
