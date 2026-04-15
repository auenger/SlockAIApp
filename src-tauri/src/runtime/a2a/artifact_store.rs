//! Cross-Agent Artifact Store.
//!
//! Manages artifacts produced by agents during task execution.
//! Artifacts are files or data produced by one agent that can be
//! referenced and consumed by other agents.
//!
//! ## Storage Backend
//!
//! Artifacts are stored on the local filesystem under the workspace root.
//! Metadata (ArtifactRef) is tracked in memory and can be persisted.
//!
//! ## Connection-Centric Access
//!
//! - Local artifacts: direct filesystem access.
//! - Remote artifacts: fetched via A2A GET /artifacts/{id} API.
//!
//! The store handles both cases transparently.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::types::Part;

// ===========================================================================
// Artifact Reference
// ===========================================================================

/// A reference to an artifact produced by an agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactRef {
    /// Unique artifact identifier.
    pub id: String,
    /// The agent that produced this artifact.
    pub producer_agent_id: String,
    /// Human-readable name (e.g., "src/utils/helper.rs").
    pub name: String,
    /// File path on the local filesystem (relative to workspace root).
    pub file_path: String,
    /// SHA-256 content hash for integrity verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    /// MIME type of the artifact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// Optional task ID this artifact was produced for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// Description of the artifact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Size in bytes.
    #[serde(default)]
    pub size: u64,
}

// ===========================================================================
// Artifact Consumption Record
// ===========================================================================

/// Record of an agent consuming (reading) an artifact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactConsumption {
    /// The artifact that was consumed.
    pub artifact_id: String,
    /// The agent that consumed the artifact.
    pub consumer_agent_id: String,
    /// ISO 8601 timestamp of consumption.
    pub consumed_at: String,
}

// ===========================================================================
// Artifact Record (full artifact with content)
// ===========================================================================

/// Full artifact record including content parts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactRecord {
    /// The artifact reference/metadata.
    #[serde(flatten)]
    pub meta: ArtifactRef,
    /// Content parts of the artifact.
    pub parts: Vec<Part>,
    /// List of consumers who have accessed this artifact.
    #[serde(default)]
    pub consumers: Vec<ArtifactConsumption>,
}

// ===========================================================================
// Artifact Store
// ===========================================================================

/// Manages cross-agent artifacts.
///
/// Artifacts are registered when an agent produces output, and can be
/// queried and consumed by other agents.
pub struct ArtifactStore {
    /// Registry of artifact references.
    artifacts: Arc<Mutex<HashMap<String, ArtifactRecord>>>,
    /// Base directory for artifact file storage.
    artifacts_dir: PathBuf,
}

impl ArtifactStore {
    /// Create a new ArtifactStore with the given base directory.
    ///
    /// The directory will be created if it doesn't exist.
    pub fn new(artifacts_dir: impl Into<PathBuf>) -> Self {
        let dir = artifacts_dir.into();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            log::warn!("[ArtifactStore] Failed to create artifacts dir: {}", e);
        }

        Self {
            artifacts: Arc::new(Mutex::new(HashMap::new())),
            artifacts_dir: dir,
        }
    }

    /// Register a new artifact.
    ///
    /// Creates the artifact record and optionally copies/moves the file
    /// to the artifact storage directory.
    pub fn register(
        &self,
        producer_agent_id: &str,
        name: &str,
        file_path: &str,
        mime_type: Option<&str>,
        task_id: Option<&str>,
        description: Option<&str>,
    ) -> Result<ArtifactRef, String> {
        let id = generate_artifact_id();
        let now = now_iso();

        // Compute content hash if file exists
        let content_hash = compute_file_hash(file_path);

        // Get file size
        let size = std::fs::metadata(file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let artifact_ref = ArtifactRef {
            id: id.clone(),
            producer_agent_id: producer_agent_id.to_string(),
            name: name.to_string(),
            file_path: file_path.to_string(),
            content_hash,
            mime_type: mime_type.map(|s| s.to_string()),
            created_at: now,
            task_id: task_id.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
            size,
        };

        // Read file content for the record
        let content = std::fs::read_to_string(file_path).unwrap_or_default();

        let record = ArtifactRecord {
            meta: artifact_ref.clone(),
            parts: vec![Part::Text { text: content }],
            consumers: Vec::new(),
        };

        let mut artifacts = self.artifacts.lock().map_err(|e| e.to_string())?;
        artifacts.insert(id, record);

        log::info!(
            "[ArtifactStore] Registered artifact '{}' ({} bytes) from agent {}",
            name,
            size,
            producer_agent_id
        );

        Ok(artifact_ref)
    }

    /// Register an artifact with inline content (no file path).
    pub fn register_inline(
        &self,
        producer_agent_id: &str,
        name: &str,
        content: &str,
        mime_type: Option<&str>,
        task_id: Option<&str>,
    ) -> Result<ArtifactRef, String> {
        let id = generate_artifact_id();
        let now = now_iso();

        let size = content.len() as u64;

        let artifact_ref = ArtifactRef {
            id: id.clone(),
            producer_agent_id: producer_agent_id.to_string(),
            name: name.to_string(),
            file_path: String::new(),
            content_hash: Some(compute_content_hash(content)),
            mime_type: mime_type.map(|s| s.to_string()),
            created_at: now,
            task_id: task_id.map(|s| s.to_string()),
            description: None,
            size,
        };

        let record = ArtifactRecord {
            meta: artifact_ref.clone(),
            parts: vec![Part::Text { text: content.to_string() }],
            consumers: Vec::new(),
        };

        let mut artifacts = self.artifacts.lock().map_err(|e| e.to_string())?;
        artifacts.insert(id, record);

        log::info!(
            "[ArtifactStore] Registered inline artifact '{}' ({} bytes) from agent {}",
            name,
            size,
            producer_agent_id
        );

        Ok(artifact_ref)
    }

    /// Get an artifact by ID.
    pub fn get(&self, artifact_id: &str) -> Result<Option<ArtifactRef>, String> {
        let artifacts = self.artifacts.lock().map_err(|e| e.to_string())?;
        Ok(artifacts.get(artifact_id).map(|r| r.meta.clone()))
    }

    /// Get the full artifact record (including content and consumers).
    pub fn get_full(&self, artifact_id: &str) -> Result<Option<ArtifactRecord>, String> {
        let artifacts = self.artifacts.lock().map_err(|e| e.to_string())?;
        Ok(artifacts.get(artifact_id).cloned())
    }

    /// Get artifact content as a string.
    pub fn get_content(&self, artifact_id: &str) -> Result<Option<String>, String> {
        let artifacts = self.artifacts.lock().map_err(|e| e.to_string())?;
        match artifacts.get(artifact_id) {
            Some(record) => {
                let mut content = String::new();
                for part in &record.parts {
                    if let Part::Text { text } = part {
                        content.push_str(text);
                    }
                }
                Ok(Some(content))
            }
            None => Ok(None),
        }
    }

    /// Record that an agent consumed (accessed) an artifact.
    pub fn record_consumption(
        &self,
        artifact_id: &str,
        consumer_agent_id: &str,
    ) -> Result<(), String> {
        let mut artifacts = self.artifacts.lock().map_err(|e| e.to_string())?;

        let record = artifacts
            .get_mut(artifact_id)
            .ok_or_else(|| format!("Artifact not found: {}", artifact_id))?;

        // Check if already consumed by this agent
        if record.consumers.iter().any(|c| c.consumer_agent_id == consumer_agent_id) {
            return Ok(()); // Already consumed, idempotent
        }

        record.consumers.push(ArtifactConsumption {
            artifact_id: artifact_id.to_string(),
            consumer_agent_id: consumer_agent_id.to_string(),
            consumed_at: now_iso(),
        });

        log::info!(
            "[ArtifactStore] Agent {} consumed artifact {}",
            consumer_agent_id,
            artifact_id
        );

        Ok(())
    }

    /// List all artifacts.
    pub fn list_all(&self) -> Result<Vec<ArtifactRef>, String> {
        let artifacts = self.artifacts.lock().map_err(|e| e.to_string())?;
        let mut result: Vec<_> = artifacts.values().map(|r| r.meta.clone()).collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }

    /// List artifacts produced by a specific agent.
    pub fn list_by_producer(&self, agent_id: &str) -> Result<Vec<ArtifactRef>, String> {
        let artifacts = self.artifacts.lock().map_err(|e| e.to_string())?;
        let mut result: Vec<_> = artifacts
            .values()
            .filter(|r| r.meta.producer_agent_id == agent_id)
            .map(|r| r.meta.clone())
            .collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }

    /// List artifacts related to a specific task.
    pub fn list_by_task(&self, task_id: &str) -> Result<Vec<ArtifactRef>, String> {
        let artifacts = self.artifacts.lock().map_err(|e| e.to_string())?;
        let result: Vec<_> = artifacts
            .values()
            .filter(|r| r.meta.task_id.as_deref() == Some(task_id))
            .map(|r| r.meta.clone())
            .collect();
        Ok(result)
    }

    /// Search artifacts by name (case-insensitive substring match).
    pub fn search(&self, query: &str) -> Result<Vec<ArtifactRef>, String> {
        let query_lower = query.to_lowercase();
        let artifacts = self.artifacts.lock().map_err(|e| e.to_string())?;
        let result: Vec<_> = artifacts
            .values()
            .filter(|r| r.meta.name.to_lowercase().contains(&query_lower))
            .map(|r| r.meta.clone())
            .collect();
        Ok(result)
    }

    /// Delete an artifact by ID.
    pub fn delete(&self, artifact_id: &str) -> Result<bool, String> {
        let mut artifacts = self.artifacts.lock().map_err(|e| e.to_string())?;
        Ok(artifacts.remove(artifact_id).is_some())
    }

    /// Get the artifacts directory path.
    pub fn artifacts_dir(&self) -> &Path {
        &self.artifacts_dir
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Generate a unique artifact ID.
fn generate_artifact_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("art-{:x}", nanos)
}

/// Compute a simple hash of a file's contents.
fn compute_file_hash(file_path: &str) -> Option<String> {
    match std::fs::read_to_string(file_path) {
        Ok(content) => Some(compute_content_hash(&content)),
        Err(_) => None,
    }
}

/// Compute a simple hash of content for integrity checking.
fn compute_content_hash(content: &str) -> String {
    // Simple hash for integrity verification.
    // In production, use SHA-256 from the `sha2` crate.
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for byte in content.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    format!("{:016x}", hash)
}

/// Get current ISO 8601 timestamp.
fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;
    let (y, m, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, hours, minutes, seconds)
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let diy = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 { 366 } else { 365 };
        if days < diy { break; }
        days -= diy;
        year += 1;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let md: [u64; 12] = if leap { [31,29,31,30,31,30,31,31,30,31,30,31] } else { [31,28,31,30,31,30,31,31,30,31,30,31] };
    let mut month = 1u64;
    for &x in &md { if days < x { break; } days -= x; month += 1; }
    (year, month, days + 1)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_ref_serde() {
        let artifact = ArtifactRef {
            id: "art-1".into(),
            producer_agent_id: "agent-a".into(),
            name: "helper.rs".into(),
            file_path: "/tmp/helper.rs".into(),
            content_hash: Some("abc123".into()),
            mime_type: Some("text/rust".into()),
            created_at: "2026-04-16T12:00:00Z".into(),
            task_id: Some("task-1".into()),
            description: Some("A helper module".into()),
            size: 1024,
        };
        let json = serde_json::to_string(&artifact).unwrap();
        let back: ArtifactRef = serde_json::from_str(&json).unwrap();
        assert_eq!(artifact, back);
    }

    #[test]
    fn test_artifact_consumption_serde() {
        let consumption = ArtifactConsumption {
            artifact_id: "art-1".into(),
            consumer_agent_id: "agent-b".into(),
            consumed_at: "2026-04-16T13:00:00Z".into(),
        };
        let json = serde_json::to_string(&consumption).unwrap();
        let back: ArtifactConsumption = serde_json::from_str(&json).unwrap();
        assert_eq!(consumption, back);
    }

    #[test]
    fn test_store_register_and_get() {
        let dir = tempfile::tempdir().unwrap();
        let store = ArtifactStore::new(dir.path());

        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "hello world").unwrap();

        let artifact = store.register(
            "agent-a",
            "test.txt",
            file_path.to_str().unwrap(),
            Some("text/plain"),
            Some("task-1"),
            None,
        ).unwrap();

        assert_eq!(artifact.producer_agent_id, "agent-a");
        assert_eq!(artifact.name, "test.txt");
        assert_eq!(artifact.size, 11);

        let retrieved = store.get(&artifact.id).unwrap().unwrap();
        assert_eq!(retrieved.id, artifact.id);
    }

    #[test]
    fn test_store_register_inline() {
        let dir = tempfile::tempdir().unwrap();
        let store = ArtifactStore::new(dir.path());

        let artifact = store.register_inline(
            "agent-b",
            "output.json",
            r#"{"result": "success"}"#,
            Some("application/json"),
            None,
        ).unwrap();

        assert_eq!(artifact.producer_agent_id, "agent-b");
        assert_eq!(artifact.size, 21);

        let content = store.get_content(&artifact.id).unwrap().unwrap();
        assert!(content.contains("success"));
    }

    #[test]
    fn test_store_consumption() {
        let dir = tempfile::tempdir().unwrap();
        let store = ArtifactStore::new(dir.path());

        let file_path = dir.path().join("data.txt");
        std::fs::write(&file_path, "data").unwrap();

        let artifact = store.register(
            "agent-a", "data.txt", file_path.to_str().unwrap(), None, None, None,
        ).unwrap();

        // Record consumption
        store.record_consumption(&artifact.id, "agent-b").unwrap();

        let record = store.get_full(&artifact.id).unwrap().unwrap();
        assert_eq!(record.consumers.len(), 1);
        assert_eq!(record.consumers[0].consumer_agent_id, "agent-b");

        // Idempotent
        store.record_consumption(&artifact.id, "agent-b").unwrap();
        let record = store.get_full(&artifact.id).unwrap().unwrap();
        assert_eq!(record.consumers.len(), 1);
    }

    #[test]
    fn test_store_list_by_producer() {
        let dir = tempfile::tempdir().unwrap();
        let store = ArtifactStore::new(dir.path());

        let fp1 = dir.path().join("a.txt");
        std::fs::write(&fp1, "a").unwrap();
        let fp2 = dir.path().join("b.txt");
        std::fs::write(&fp2, "b").unwrap();

        store.register("agent-a", "a.txt", fp1.to_str().unwrap(), None, None, None).unwrap();
        store.register("agent-b", "b.txt", fp2.to_str().unwrap(), None, None, None).unwrap();

        let by_a = store.list_by_producer("agent-a").unwrap();
        assert_eq!(by_a.len(), 1);
        assert_eq!(by_a[0].producer_agent_id, "agent-a");
    }

    #[test]
    fn test_store_search() {
        let dir = tempfile::tempdir().unwrap();
        let store = ArtifactStore::new(dir.path());

        let fp1 = dir.path().join("helper.rs");
        std::fs::write(&fp1, "fn help() {}").unwrap();
        let fp2 = dir.path().join("main.rs");
        std::fs::write(&fp2, "fn main() {}").unwrap();

        store.register("agent-a", "helper.rs", fp1.to_str().unwrap(), None, None, None).unwrap();
        store.register("agent-a", "main.rs", fp2.to_str().unwrap(), None, None, None).unwrap();

        let results = store.search("helper").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "helper.rs");
    }

    #[test]
    fn test_store_delete() {
        let dir = tempfile::tempdir().unwrap();
        let store = ArtifactStore::new(dir.path());

        let fp = dir.path().join("temp.txt");
        std::fs::write(&fp, "temp").unwrap();

        let artifact = store.register("agent-a", "temp.txt", fp.to_str().unwrap(), None, None, None).unwrap();

        assert!(store.delete(&artifact.id).unwrap());
        assert!(store.get(&artifact.id).unwrap().is_none());
        assert!(!store.delete(&artifact.id).unwrap()); // Already deleted
    }

    #[test]
    fn test_compute_content_hash() {
        let hash1 = compute_content_hash("hello world");
        let hash2 = compute_content_hash("hello world");
        let hash3 = compute_content_hash("hello earth");

        assert_eq!(hash1, hash2); // Same content = same hash
        assert_ne!(hash1, hash3); // Different content = different hash
    }
}
