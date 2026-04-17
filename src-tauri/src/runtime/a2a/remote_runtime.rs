//! Remote A2A Runtime -- implements AgentRuntime trait for remote endpoints.
//!
//! This runtime connects to a remote A2A-compatible endpoint via HTTP,
//! creating tasks, sending messages, and streaming responses back as
//! StreamEvent instances that are identical to local CLI runtime events.

use std::sync::mpsc::Receiver;

use super::transport::{A2AHttpClient, A2ATransport};
use super::types::{Message, RemoteConnection};
use crate::runtime::{
    AgentCapability, AgentRuntime, AgentRuntimeInfo, AgentRuntimeStatus, ExecuteParams,
    RuntimeType, StreamEvent,
};
use crate::storage::keyring;

/// A remote A2A agent runtime.
///
/// Connects to a remote A2A endpoint via HTTP and implements the
/// `AgentRuntime` trait so that remote agents are treated identically
/// to local CLI-based agents from the frontend's perspective.
pub struct RemoteA2ARuntime {
    /// The remote connection this runtime is bound to.
    connection: RemoteConnection,
}

impl RemoteA2ARuntime {
    /// Create a new RemoteA2ARuntime for the given connection.
    pub fn new(connection: RemoteConnection) -> Self {
        Self { connection }
    }

    /// Build an A2A HTTP client for this connection, injecting auth token.
    fn build_a2a_client(&self) -> Result<A2AHttpClient, String> {
        let token = self.get_auth_token()?;

        match token {
            Some(t) => Ok(A2AHttpClient::with_bearer_token(
                &self.connection.endpoint_url,
                &t,
            )),
            None => Ok(A2AHttpClient::new(&self.connection.endpoint_url)),
        }
    }

    /// Get the auth token from keyring for this connection.
    fn get_auth_token(&self) -> Result<Option<String>, String> {
        let keyring_key = format!("remote_conn_{}", self.connection.id);
        keyring::get_api_key_internal(&keyring_key)
    }

    /// Generate a unique task ID.
    fn generate_task_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("task-{:x}", nanos)
    }
}

impl AgentRuntime for RemoteA2ARuntime {
    fn id(&self) -> &str {
        // Use the connection ID as the runtime ID
        &self.connection.id
    }

    fn name(&self) -> &str {
        &self.connection.name
    }

    fn runtime_category(&self) -> &str {
        "http"
    }

    fn typed_runtime_type(&self) -> RuntimeType {
        RuntimeType::Custom("remote-a2a".to_string())
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        // Remote A2A supports streaming (via SSE)
        vec![AgentCapability::Streaming, AgentCapability::ToolUse]
    }

    fn install_hint(&self) -> String {
        "Configure a remote A2A endpoint in Settings > Remote Connections".to_string()
    }

    fn binary_name(&self) -> &str {
        "" // No local binary for remote runtime
    }

    fn detect(&self) -> Result<Option<(String, String)>, String> {
        // For remote, "detection" means checking if the connection is online
        match self.connection.status {
            super::types::ConnectionStatus::Online => {
                Ok(Some((
                    self.connection.endpoint_url.clone(),
                    "remote-a2a".to_string(),
                )))
            }
            _ => Ok(None),
        }
    }

    fn health_check(&self) -> AgentRuntimeStatus {
        match self.connection.status {
            super::types::ConnectionStatus::Online => AgentRuntimeStatus::Available,
            super::types::ConnectionStatus::Offline => AgentRuntimeStatus::NotInstalled,
            super::types::ConnectionStatus::Error => AgentRuntimeStatus::Unhealthy,
            super::types::ConnectionStatus::Unknown => AgentRuntimeStatus::Detecting,
        }
    }

    fn info(&self) -> AgentRuntimeInfo {
        let status = self.health_check();
        let detected = self.detect().ok().flatten();

        AgentRuntimeInfo {
            id: self.connection.id.clone(),
            name: self.connection.name.clone(),
            runtime_category: "http".to_string(),
            runtime_type: self.typed_runtime_type(),
            status: status.as_str().to_string(),
            version: detected.as_ref().map(|(_, v)| v.clone()),
            install_path: detected.as_ref().map(|(u, _)| u.clone()),
            capabilities: self.capabilities(),
            install_hint: self.install_hint(),
            binary_name: None,
        }
    }

    fn is_ready(&self) -> bool {
        matches!(
            self.connection.status,
            super::types::ConnectionStatus::Online
        )
    }

    fn execute(&self, params: ExecuteParams) -> Result<Receiver<StreamEvent>, String> {
        let client = self.build_a2a_client()?;

        let task_id = match &params.session_id {
            Some(sid) if params.persistent => {
                // Thread mode: reuse session as task context
                log::info!(
                    "[RemoteA2ARuntime] Resuming session on '{}' (session: {})",
                    self.connection.name,
                    sid
                );
                sid.clone()
            }
            _ => Self::generate_task_id(),
        };

        log::info!(
            "[RemoteA2ARuntime] Executing on '{}' (task: {}): {}",
            self.connection.name,
            task_id,
            &params.message[..params.message.len().min(80)]
        );

        // Build user message with optional system context
        let mut message = Message::user_text(&params.message);

        // Pass system_prompt as message metadata so remote agent has context
        if let Some(ref prompt) = params.system_prompt {
            message.metadata = Some(serde_json::json!({
                "system_context": prompt,
                "agent_id": params.agent_id,
                "persistent": params.persistent,
                "workspace": params.workspace,
            }));
        }

        // Use streaming via SSE
        let receiver = client
            .stream_message(&task_id, message)
            .map_err(|e| format!("Remote execution failed: {}", e))?;

        Ok(receiver)
    }
}
