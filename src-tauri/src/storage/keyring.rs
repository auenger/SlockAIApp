//! Secure API key storage using OS Keyring.
//!
//! API Keys are stored in the OS keychain (Keychain on macOS,
//! Credential Manager on Windows, Secret Service on Linux).
//! Keys are never exposed to the frontend - only stored/checked/deleted.

/// Service name used for keyring entries.
const SERVICE_NAME: &str = "AgentsZone";

/// Store an API key for a given runtime in the OS keyring.
#[tauri::command]
pub fn store_api_key(runtime_id: String, api_key: String) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE_NAME, &runtime_id)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    entry
        .set_password(&api_key)
        .map_err(|e| format!("Failed to store API key: {}", e))?;
    log::info!("[keyring] API key stored for runtime: {}", runtime_id);
    Ok(())
}

/// Check if an API key exists for a given runtime.
/// Returns true if a key is stored, false otherwise.
/// Does NOT return the key itself.
#[tauri::command]
pub fn has_api_key(runtime_id: String) -> Result<bool, String> {
    let entry = keyring::Entry::new(SERVICE_NAME, &runtime_id)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(format!("Failed to check API key: {}", e)),
    }
}

/// Delete an API key for a given runtime from the OS keyring.
#[tauri::command]
pub fn delete_api_key(runtime_id: String) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE_NAME, &runtime_id)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete API key: {}", e))?;
    log::info!("[keyring] API key deleted for runtime: {}", runtime_id);
    Ok(())
}

/// Internal helper: retrieve an API key for use in Rust backend only.
/// This is NOT a Tauri command - it should never be exposed to the frontend.
pub fn get_api_key_internal(runtime_id: &str) -> Result<Option<String>, String> {
    let entry = keyring::Entry::new(SERVICE_NAME, runtime_id)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to get API key: {}", e)),
    }
}
