//! Runtime Registry implementation.
//!
//! Thread-safe registry for managing agent runtimes.
//! Handles registration, detection, querying, and health checking.

use super::{AgentRuntime, AgentRuntimeInfo, AgentRuntimeStatus};
use std::collections::HashMap;

// ===========================================================================
// RuntimeRegistry
// ===========================================================================

/// Registry that manages all discovered agent runtimes.
#[derive(Default)]
pub struct RuntimeRegistry {
    runtimes: Vec<Box<dyn AgentRuntime>>,
    /// Cached detection results: runtime_id -> (path, version)
    detected: HashMap<String, (String, String)>,
}

impl RuntimeRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            runtimes: Vec::new(),
            detected: HashMap::new(),
        }
    }

    /// Register a runtime implementation.
    pub fn register(&mut self, runtime: Box<dyn AgentRuntime>) {
        self.runtimes.push(runtime);
    }

    /// Scan all registered runtimes, detect their presence on the system.
    /// Returns info for each runtime.
    pub fn scan_all(&mut self) -> Vec<AgentRuntimeInfo> {
        self.detected.clear();
        let mut results = Vec::new();

        for runtime in &self.runtimes {
            let rt_id = runtime.id().to_string();
            match runtime.detect() {
                Ok(Some((path, version))) => {
                    self.detected
                        .insert(rt_id.clone(), (path.clone(), version.clone()));
                    results.push(AgentRuntimeInfo {
                        id: runtime.id().to_string(),
                        name: runtime.name().to_string(),
                        runtime_type: runtime.runtime_type().to_string(),
                        status: AgentRuntimeStatus::Available.as_str().to_string(),
                        version: Some(version),
                        install_path: Some(path),
                        capabilities: runtime.capabilities(),
                        install_hint: runtime.install_hint(),
                    });
                }
                Ok(None) => {
                    results.push(AgentRuntimeInfo {
                        id: runtime.id().to_string(),
                        name: runtime.name().to_string(),
                        runtime_type: runtime.runtime_type().to_string(),
                        status: AgentRuntimeStatus::NotInstalled.as_str().to_string(),
                        version: None,
                        install_path: None,
                        capabilities: runtime.capabilities(),
                        install_hint: runtime.install_hint(),
                    });
                }
                Err(_) => {
                    results.push(AgentRuntimeInfo {
                        id: runtime.id().to_string(),
                        name: runtime.name().to_string(),
                        runtime_type: runtime.runtime_type().to_string(),
                        status: AgentRuntimeStatus::NotInstalled.as_str().to_string(),
                        version: None,
                        install_path: None,
                        capabilities: runtime.capabilities(),
                        install_hint: runtime.install_hint(),
                    });
                }
            }
        }

        results
    }

    /// Get runtime info for all registered runtimes (using cached detection data).
    pub fn list_all(&self) -> Vec<AgentRuntimeInfo> {
        self.runtimes
            .iter()
            .map(|rt| {
                let rt_id = rt.id().to_string();
                if let Some((path, version)) = self.detected.get(&rt_id) {
                    AgentRuntimeInfo {
                        id: rt.id().to_string(),
                        name: rt.name().to_string(),
                        runtime_type: rt.runtime_type().to_string(),
                        status: AgentRuntimeStatus::Available.as_str().to_string(),
                        version: Some(version.clone()),
                        install_path: Some(path.clone()),
                        capabilities: rt.capabilities(),
                        install_hint: rt.install_hint(),
                    }
                } else {
                    AgentRuntimeInfo {
                        id: rt.id().to_string(),
                        name: rt.name().to_string(),
                        runtime_type: rt.runtime_type().to_string(),
                        status: AgentRuntimeStatus::NotInstalled.as_str().to_string(),
                        version: None,
                        install_path: None,
                        capabilities: rt.capabilities(),
                        install_hint: rt.install_hint(),
                    }
                }
            })
            .collect()
    }

    /// Get info for a single runtime by id.
    pub fn get_runtime(&self, id: &str) -> Option<AgentRuntimeInfo> {
        self.runtimes
            .iter()
            .find(|rt| rt.id() == id)
            .map(|rt| {
                let rt_id = rt.id().to_string();
                if let Some((path, version)) = self.detected.get(&rt_id) {
                    AgentRuntimeInfo {
                        id: rt.id().to_string(),
                        name: rt.name().to_string(),
                        runtime_type: rt.runtime_type().to_string(),
                        status: AgentRuntimeStatus::Available.as_str().to_string(),
                        version: Some(version.clone()),
                        install_path: Some(path.clone()),
                        capabilities: rt.capabilities(),
                        install_hint: rt.install_hint(),
                    }
                } else {
                    AgentRuntimeInfo {
                        id: rt.id().to_string(),
                        name: rt.name().to_string(),
                        runtime_type: rt.runtime_type().to_string(),
                        status: AgentRuntimeStatus::NotInstalled.as_str().to_string(),
                        version: None,
                        install_path: None,
                        capabilities: rt.capabilities(),
                        install_hint: rt.install_hint(),
                    }
                }
            })
    }

    /// Get a reference to a registered runtime instance by id.
    /// Returns a reference to the trait object for calling execute() etc.
    pub fn get_runtime_instance(&self, id: &str) -> Result<&dyn AgentRuntime, String> {
        self.runtimes
            .iter()
            .find(|rt| rt.id() == id)
            .map(|rt| rt.as_ref())
            .ok_or_else(|| format!("Runtime '{}' not found in registry", id))
    }

    /// Run health checks on all detected runtimes.
    pub fn health_check_all(&mut self) -> Vec<AgentRuntimeInfo> {
        let mut results = Vec::new();
        for runtime in &self.runtimes {
            let status = runtime.health_check();
            let rt_id = runtime.id().to_string();
            let (version, install_path) = self
                .detected
                .get(&rt_id)
                .map(|(p, v)| (Some(v.clone()), Some(p.clone())))
                .unwrap_or((None, None));

            // If health check says unhealthy, update detected map
            if status == AgentRuntimeStatus::Unhealthy {
                self.detected.remove(&rt_id);
            }

            results.push(AgentRuntimeInfo {
                id: rt_id,
                name: runtime.name().to_string(),
                runtime_type: runtime.runtime_type().to_string(),
                status: status.as_str().to_string(),
                version,
                install_path,
                capabilities: runtime.capabilities(),
                install_hint: runtime.install_hint(),
            });
        }
        results
    }

    /// Count available (detected & healthy) runtimes.
    pub fn available_count(&self) -> usize {
        self.detected.len()
    }
}

/// Create the default registry with Claude Code runtime registered.
pub fn create_default_registry() -> RuntimeRegistry {
    let mut registry = RuntimeRegistry::new();
    registry.register(Box::new(super::claude::ClaudeCodeRuntime::new()));
    registry
}
