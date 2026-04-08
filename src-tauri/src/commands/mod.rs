/// Tauri IPC command handlers.
///
/// Each command handles a specific domain of IPC calls from the frontend.
/// Commands are registered in lib.rs via `invoke_handler`.

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to SlockAI.", name)
}
