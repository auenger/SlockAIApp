//! Secure API key storage using OS Keyring.
//!
//! API Keys are stored in the OS keychain (Keychain on macOS,
//! Credential Manager on Windows, Secret Service on Linux).
//! Keys are never exposed to the frontend - only stored/checked/deleted.
//! The list command returns masked keys for display purposes.

use serde::Serialize;

/// Service name used for keyring entries.
const SERVICE_NAME: &str = "AgentsZone";

/// Known provider/runtime IDs that the app manages keys for.
const KNOWN_PROVIDERS: &[&str] = &[
    "claude-code",
    "openai",
    "anthropic",
    "gemini",
];

/// Masked API key info returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct ApiKeyInfo {
    /// The runtime/provider identifier (e.g. "claude-code").
    pub id: String,
    /// Human-readable name for the provider.
    pub name: String,
    /// Masked key for display (e.g. "sk-***...xyz").
    pub masked_key: String,
    /// Whether the key is currently stored.
    pub has_key: bool,
}

/// Mask an API key for safe display.
/// Returns a string like "sk-***...xyz" showing only first 3 and last 3 chars.
fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        // Too short to meaningfully mask - just show stars
        "***".to_string()
    } else {
        let prefix = &key[..3];
        let suffix = &key[key.len() - 3..];
        format!("{}***...{}", prefix, suffix)
    }
}

/// Get a human-readable name for a provider ID.
fn provider_name(id: &str) -> String {
    match id {
        "claude-code" => "Claude Code".to_string(),
        "openai" => "OpenAI".to_string(),
        "anthropic" => "Anthropic".to_string(),
        "gemini" => "Gemini".to_string(),
        _ => id.to_string(),
    }
}

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

/// List all known API keys with masked values for display.
/// Iterates over known providers and checks if a key exists for each.
#[tauri::command]
pub fn list_api_keys() -> Result<Vec<ApiKeyInfo>, String> {
    let mut keys = Vec::new();

    for &provider_id in KNOWN_PROVIDERS {
        let entry = keyring::Entry::new(SERVICE_NAME, provider_id)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

        match entry.get_password() {
            Ok(key) => {
                keys.push(ApiKeyInfo {
                    id: provider_id.to_string(),
                    name: provider_name(provider_id),
                    masked_key: mask_key(&key),
                    has_key: true,
                });
            }
            Err(keyring::Error::NoEntry) => {
                keys.push(ApiKeyInfo {
                    id: provider_id.to_string(),
                    name: provider_name(provider_id),
                    masked_key: String::new(),
                    has_key: false,
                });
            }
            Err(e) => {
                log::warn!("[keyring] Error checking key for {}: {}", provider_id, e);
                keys.push(ApiKeyInfo {
                    id: provider_id.to_string(),
                    name: provider_name(provider_id),
                    masked_key: String::new(),
                    has_key: false,
                });
            }
        }
    }

    Ok(keys)
}

/// Verify that an API key is valid by checking it exists and is non-empty.
/// Returns the masked key if valid, or an error if not.
#[tauri::command]
pub fn verify_api_key(runtime_id: String) -> Result<ApiKeyInfo, String> {
    let entry = keyring::Entry::new(SERVICE_NAME, &runtime_id)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.get_password() {
        Ok(key) => {
            if key.is_empty() {
                Ok(ApiKeyInfo {
                    id: runtime_id.clone(),
                    name: provider_name(&runtime_id),
                    masked_key: String::new(),
                    has_key: false,
                })
            } else {
                Ok(ApiKeyInfo {
                    id: runtime_id.clone(),
                    name: provider_name(&runtime_id),
                    masked_key: mask_key(&key),
                    has_key: true,
                })
            }
        }
        Err(keyring::Error::NoEntry) => Ok(ApiKeyInfo {
            id: runtime_id.clone(),
            name: provider_name(&runtime_id),
            masked_key: String::new(),
            has_key: false,
        }),
        Err(e) => Err(format!("Failed to verify API key: {}", e)),
    }
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
