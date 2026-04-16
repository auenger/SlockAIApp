//! Bridge extension protocol handlers.
//!
//! Implements the `bridge.*` JSON-RPC methods for remote workspace management:
//! - `bridge.getWorkspaceInfo` — workspace metadata
//! - `bridge.getAgents` — remote agent list
//! - `bridge.listFiles` — workspace file browsing
//! - `bridge.readFile` — remote file content reading

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::runtime::a2a::adapter::handler::AdapterServer;
use crate::runtime::a2a::types::A2AError;
use crate::workspace::manager::AgentManager;

// ===========================================================================
// Response types
// ===========================================================================

#[derive(Debug, Serialize)]
struct WorkspaceInfoResponse {
    workspace_root: String,
    total_agents: usize,
    enabled_agents: usize,
    active_agent_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct AgentInfo {
    agent_id: String,
    name: String,
    emoji: String,
    creature: String,
    vibe: String,
    runtime_type: String,
}

#[derive(Debug, Serialize)]
struct AgentsResponse {
    agents: Vec<AgentInfo>,
}

#[derive(Debug, Serialize)]
struct FileEntry {
    name: String,
    is_dir: bool,
    size: u64,
    modified: u64,
}

#[derive(Debug, Serialize)]
struct ListFilesResponse {
    entries: Vec<FileEntry>,
}

#[derive(Debug, Serialize)]
struct ReadFileResponse {
    name: String,
    size: u64,
    mime_type: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AgentIdParam {
    agent_id: String,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FilePathParam {
    agent_id: String,
    file_path: String,
}

// ===========================================================================
// Handler registration
// ===========================================================================

/// Register all bridge.* extension handlers on the AdapterServer.
pub fn register_bridge_handlers(
    server: &AdapterServer,
    agent_manager: &Arc<Mutex<AgentManager>>,
) {
    let mgr = agent_manager.clone();

    // --- bridge.getWorkspaceInfo ---
    server.register_handler("bridge.getWorkspaceInfo", move |_| {
        let mgr = mgr.lock().unwrap();
        let status = mgr.get_status();

        Ok(serde_json::to_value(WorkspaceInfoResponse {
            workspace_root: status.workspace_root,
            total_agents: status.total_agents,
            enabled_agents: status.enabled_agents,
            active_agent_id: status.active_agent_id,
        })
        .unwrap_or_default())
    });

    // --- bridge.getAgents ---
    let mgr_agents = agent_manager.clone();
    server.register_handler("bridge.getAgents", move |_| {
        let mgr = mgr_agents.lock().unwrap();
        let agents: Vec<AgentInfo> = mgr
            .list_agents()
            .into_iter()
            .map(|a| AgentInfo {
                agent_id: a.agent_id,
                name: a.name,
                emoji: a.emoji,
                creature: "".to_string(), // not in summary
                vibe: "".to_string(),     // not in summary
                runtime_type: format!("{:?}", a.runtime_type),
            })
            .collect();

        Ok(serde_json::to_value(AgentsResponse { agents }).unwrap_or_default())
    });

    // --- bridge.listFiles ---
    let mgr_files = agent_manager.clone();
    server.register_handler("bridge.listFiles", move |params| {
        let req: AgentIdParam = serde_json::from_value(params)
            .map_err(|e| A2AError::invalid_params(format!("Invalid params: {}", e)))?;

        let mgr = mgr_files.lock().unwrap();
        let workspace = mgr
            .get_workspace(&req.agent_id)
            .ok_or_else(|| A2AError::invalid_params(format!("Agent not found: {}", req.agent_id)))?;

        let base_path = workspace.base_path().to_path_buf();
        let target_path = match &req.path {
            Some(rel_path) => {
                let clean = sanitize_path(rel_path)?;
                let target = base_path.join(&clean);
                // Security: verify within workspace
                verify_path_within(&target, &base_path)?;
                target
            }
            None => base_path.clone(),
        };

        if !target_path.is_dir() {
            return Err(A2AError::invalid_params("Not a directory"));
        }

        let entries = list_dir_entries(&target_path)?;

        Ok(serde_json::to_value(ListFilesResponse { entries }).unwrap_or_default())
    });

    // --- bridge.readFile ---
    let mgr_read = agent_manager.clone();
    server.register_handler("bridge.readFile", move |params| {
        let req: FilePathParam = serde_json::from_value(params)
            .map_err(|e| A2AError::invalid_params(format!("Invalid params: {}", e)))?;

        let mgr = mgr_read.lock().unwrap();
        let workspace = mgr
            .get_workspace(&req.agent_id)
            .ok_or_else(|| A2AError::invalid_params(format!("Agent not found: {}", req.agent_id)))?;

        let base_path = workspace.base_path().to_path_buf();
        let clean = sanitize_path(&req.file_path)?;
        let full_path = base_path.join(&clean);

        // Security: verify within workspace
        verify_path_within(&full_path, &base_path)?;

        if !full_path.is_file() {
            return Err(A2AError::invalid_params("Not a file"));
        }

        let metadata = fs::metadata(&full_path)
            .map_err(|e| A2AError::internal_error(format!("Metadata failed: {}", e)))?;

        let content = fs::read_to_string(&full_path)
            .map_err(|e| A2AError::internal_error(format!("Read failed: {}", e)))?;

        let name = full_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let mime_type = mime_guess::from_path(&full_path)
            .first_or_octet_stream()
            .to_string();

        Ok(serde_json::to_value(ReadFileResponse {
            name,
            size: metadata.len(),
            mime_type,
            content,
        })
        .unwrap_or_default())
    });
}

// ===========================================================================
// Security helpers
// ===========================================================================

/// Sanitize a relative path to prevent path traversal.
fn sanitize_path(path: &str) -> Result<String, A2AError> {
    // Reject obvious traversal patterns
    if path.contains("..") {
        return Err(A2AError::invalid_params(
            "Invalid path: traversal not allowed",
        ));
    }

    // Normalize path separators
    let clean = path.replace('\\', "/");

    // Reject absolute paths
    if clean.starts_with('/') {
        return Err(A2AError::invalid_params(
            "Invalid path: absolute paths not allowed",
        ));
    }

    Ok(clean)
}

/// Verify that a path is within the allowed base directory.
fn verify_path_within(target: &PathBuf, base: &PathBuf) -> Result<(), A2AError> {
    let canonical_target = target
        .canonicalize()
        .or_else(|_| {
            // If target doesn't exist yet, use the parent
            target.parent()
                .and_then(|p| p.canonicalize().ok())
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "path not found"))
        })
        .map_err(|_| A2AError::invalid_params("Invalid path"))?;

    let canonical_base = base
        .canonicalize()
        .map_err(|_| A2AError::internal_error("Cannot resolve base path"))?;

    if !canonical_target.starts_with(&canonical_base) {
        return Err(A2AError::invalid_params(
            "Access denied: path outside workspace",
        ));
    }

    Ok(())
}

/// List directory entries sorted by directories first, then files.
fn list_dir_entries(dir: &PathBuf) -> Result<Vec<FileEntry>, A2AError> {
    let entries = fs::read_dir(dir)
        .map_err(|e| A2AError::internal_error(format!("Read dir failed: {}", e)))?;

    let mut result: Vec<FileEntry> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = metadata.is_dir();
            let size = if is_dir { 0 } else { metadata.len() };
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            Some(FileEntry {
                name,
                is_dir,
                size,
                modified,
            })
        })
        .collect();

    result.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(result)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path_normal() {
        assert_eq!(sanitize_path("foo/bar.md").unwrap(), "foo/bar.md");
    }

    #[test]
    fn test_sanitize_path_traversal_rejected() {
        assert!(sanitize_path("../etc/passwd").is_err());
        assert!(sanitize_path("foo/../../etc/passwd").is_err());
        assert!(sanitize_path("..").is_err());
    }

    #[test]
    fn test_sanitize_path_absolute_rejected() {
        assert!(sanitize_path("/etc/passwd").is_err());
    }

    #[test]
    fn test_sanitize_path_backslash_normalized() {
        assert_eq!(sanitize_path("foo\\bar.md").unwrap(), "foo/bar.md");
    }

    #[test]
    fn test_list_dir_entries() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "hello").unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("b.md"), "# heading").unwrap();

        let entries = list_dir_entries(&dir.path().to_path_buf()).unwrap();

        assert_eq!(entries.len(), 3);
        // Directories first
        assert!(entries[0].is_dir);
        assert_eq!(entries[0].name, "subdir");
        // Then files sorted alphabetically
        assert!(!entries[1].is_dir);
        assert_eq!(entries[1].name, "a.txt");
    }

    #[test]
    fn test_verify_path_within_allowed() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("agents/default")).unwrap();
        let base = dir.path().to_path_buf();
        let target = dir.path().join("agents/default/IDENTITY.md");

        // Path within workspace is OK (even if file doesn't exist,
        // parent directory canonicalization works)
        assert!(verify_path_within(&target, &base).is_ok());
    }

    #[test]
    fn test_verify_path_within_blocked() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("agents")).unwrap();
        let base = dir.path().to_path_buf();
        let target = std::path::PathBuf::from("/etc/passwd");

        assert!(verify_path_within(&target, &base).is_err());
    }
}
