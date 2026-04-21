//! Runtime Registry implementation.
//!
//! Thread-safe registry for managing agent runtimes.
//! Handles registration, detection, querying, and health checking.
//! Also provides runtime resolution that routes local agents to registered
//! runtimes and remote agents to dynamically-created RemoteA2ARuntime instances.

use super::a2a::types::ConnectionMode;
use super::{AgentRuntime, AgentRuntimeInfo, AgentRuntimeStatus, RuntimeType};
use std::collections::HashMap;

#[cfg(feature = "tauri-app")]
use super::a2a::remote_runtime::RemoteA2ARuntime;
#[cfg(feature = "tauri-app")]
use super::a2a::types::ConnectionStatus;

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
                        runtime_category: runtime.runtime_category().to_string(),
                        runtime_type: runtime.typed_runtime_type(),
                        status: AgentRuntimeStatus::Available.as_str().to_string(),
                        version: Some(version),
                        install_path: Some(path),
                        capabilities: runtime.capabilities(),
                        install_hint: runtime.install_hint(),
                        binary_name: Some(runtime.binary_name().to_string()),
                    });
                }
                Ok(None) => {
                    results.push(AgentRuntimeInfo {
                        id: runtime.id().to_string(),
                        name: runtime.name().to_string(),
                        runtime_category: runtime.runtime_category().to_string(),
                        runtime_type: runtime.typed_runtime_type(),
                        status: AgentRuntimeStatus::NotInstalled.as_str().to_string(),
                        version: None,
                        install_path: None,
                        capabilities: runtime.capabilities(),
                        install_hint: runtime.install_hint(),
                        binary_name: Some(runtime.binary_name().to_string()),
                    });
                }
                Err(_) => {
                    results.push(AgentRuntimeInfo {
                        id: runtime.id().to_string(),
                        name: runtime.name().to_string(),
                        runtime_category: runtime.runtime_category().to_string(),
                        runtime_type: runtime.typed_runtime_type(),
                        status: AgentRuntimeStatus::NotInstalled.as_str().to_string(),
                        version: None,
                        install_path: None,
                        capabilities: runtime.capabilities(),
                        install_hint: runtime.install_hint(),
                        binary_name: Some(runtime.binary_name().to_string()),
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
                        runtime_category: rt.runtime_category().to_string(),
                        runtime_type: rt.typed_runtime_type(),
                        status: AgentRuntimeStatus::Available.as_str().to_string(),
                        version: Some(version.clone()),
                        install_path: Some(path.clone()),
                        capabilities: rt.capabilities(),
                        install_hint: rt.install_hint(),
                        binary_name: Some(rt.binary_name().to_string()),
                    }
                } else {
                    AgentRuntimeInfo {
                        id: rt.id().to_string(),
                        name: rt.name().to_string(),
                        runtime_category: rt.runtime_category().to_string(),
                        runtime_type: rt.typed_runtime_type(),
                        status: AgentRuntimeStatus::NotInstalled.as_str().to_string(),
                        version: None,
                        install_path: None,
                        capabilities: rt.capabilities(),
                        install_hint: rt.install_hint(),
                        binary_name: Some(rt.binary_name().to_string()),
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
                        runtime_category: rt.runtime_category().to_string(),
                        runtime_type: rt.typed_runtime_type(),
                        status: AgentRuntimeStatus::Available.as_str().to_string(),
                        version: Some(version.clone()),
                        install_path: Some(path.clone()),
                        capabilities: rt.capabilities(),
                        install_hint: rt.install_hint(),
                        binary_name: Some(rt.binary_name().to_string()),
                    }
                } else {
                    AgentRuntimeInfo {
                        id: rt.id().to_string(),
                        name: rt.name().to_string(),
                        runtime_category: rt.runtime_category().to_string(),
                        runtime_type: rt.typed_runtime_type(),
                        status: AgentRuntimeStatus::NotInstalled.as_str().to_string(),
                        version: None,
                        install_path: None,
                        capabilities: rt.capabilities(),
                        install_hint: rt.install_hint(),
                        binary_name: Some(rt.binary_name().to_string()),
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
                runtime_category: runtime.runtime_category().to_string(),
                runtime_type: runtime.typed_runtime_type(),
                status: status.as_str().to_string(),
                version,
                install_path,
                capabilities: runtime.capabilities(),
                install_hint: runtime.install_hint(),
                binary_name: Some(runtime.binary_name().to_string()),
            });
        }
        results
    }

    /// Count available (detected & healthy) runtimes.
    pub fn available_count(&self) -> usize {
        self.detected.len()
    }

    /// Get info for a runtime by its `RuntimeType` enum variant.
    pub fn get_runtime_by_type(&self, runtime_type: &RuntimeType) -> Option<AgentRuntimeInfo> {
        let target_id = runtime_type.runtime_id();
        self.get_runtime(target_id)
    }
}

/// Create the default registry with all known runtime implementations registered.
pub fn create_default_registry() -> RuntimeRegistry {
    let mut registry = RuntimeRegistry::new();
    registry.register(Box::new(super::claude::ClaudeCodeRuntime::new()));
    registry.register(Box::new(super::codex::CodexRuntime::new()));
    registry
}

// ===========================================================================
// Runtime Resolution — routes local vs remote agents
// ===========================================================================

/// Resolved runtime for agent execution.
///
/// Abstracts over whether the agent runs locally or remotely.
#[cfg(feature = "tauri-app")]
pub enum ResolvedRuntime {
    /// Agent runs via a locally registered runtime (Claude Code, Codex, etc.).
    Local {
        /// Reference to the runtime in the registry.
        /// The string is the runtime_id used for registry lookup.
        runtime_id: String,
    },
    /// Agent runs on a remote A2A endpoint.
    Remote {
        /// The dynamically-created remote runtime instance.
        runtime: RemoteA2ARuntime,
    },
}

/// Resolve the appropriate runtime for an agent based on its connection_mode.
///
/// - `Local` agents: returns the runtime_id for registry lookup
/// - `Remote` agents: loads the connection from SQLite and creates a RemoteA2ARuntime
///
/// Returns an error if the agent is remote but the connection is not found or offline.
#[cfg(feature = "tauri-app")]
pub fn resolve_runtime_for_agent(
    connection_mode: &ConnectionMode,
    db_conn: &rusqlite::Connection,
) -> Result<ResolvedRuntime, String> {
    match connection_mode {
        ConnectionMode::Local => {
            // Local agent: the caller should use runtime_type to look up the registry
            // We return Local variant and let the caller handle registry lookup
            Ok(ResolvedRuntime::Local {
                runtime_id: String::new(), // Caller fills this from agent.runtime_type
            })
        }
        ConnectionMode::Remote { connection_id } => {
            // Remote agent: load connection from DB and create RemoteA2ARuntime
            let row = crate::storage::db_helpers::get_remote_connection(db_conn, connection_id)
                .map_err(|e| format!("Failed to load remote connection: {}", e))?
                .ok_or_else(|| {
                    format!(
                        "远程连接 '{}' 不存在。请在设置中检查 Remote Connections 配置。",
                        connection_id
                    )
                })?;

            let conn = crate::runtime::a2a::remote::row_to_connection(&row);

            // Check connection status
            if conn.status != ConnectionStatus::Online {
                return Err(format!(
                    "{} 当前不可用（远程连接{}）。请检查连接状态或重新进行健康检查。",
                    conn.name,
                    match conn.status {
                        ConnectionStatus::Offline => "已断开",
                        ConnectionStatus::Error => "出错",
                        ConnectionStatus::Unknown => "状态未知",
                        _ => "",
                    }
                ));
            }

            let runtime = RemoteA2ARuntime::new(conn);

            Ok(ResolvedRuntime::Remote { runtime })
        }
    }
}
