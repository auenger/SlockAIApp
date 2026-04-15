//! A2A (Agent-to-Agent) protocol type definitions.
//!
//! Implements the Google A2A protocol v1.0.0 data model including:
//! - Task lifecycle (status machine, messages, artifacts)
//! - Message / Part / Artifact content types
//! - AgentCard self-description
//! - JSON-RPC request/response wrappers
//!
//! Also defines the Connection-Centric model types for local vs remote agents:
//! - ConnectionMode (Local / Remote)
//! - RemoteConnection (endpoint configuration)
//! - AuthType / ConnectionStatus

use serde::{Deserialize, Serialize};

// ===========================================================================
// Task lifecycle
// ===========================================================================

/// Status of an A2A Task, following the standard state machine:
/// `SUBMITTED -> WORKING -> COMPLETED | FAILED | CANCELED | REJECTED |
///                         INPUT_REQUIRED | AUTH_REQUIRED`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    /// Task has been submitted but not yet picked up.
    #[default]
    Submitted,
    /// Task is actively being processed.
    Working,
    /// Task completed successfully.
    Completed,
    /// Task failed due to an error.
    Failed,
    /// Task was cancelled by the user or system.
    Canceled,
    /// Task was rejected by the agent.
    Rejected,
    /// Agent needs additional user input to proceed.
    InputRequired,
    /// Agent requires authentication to proceed.
    AuthRequired,
}

impl TaskStatus {
    /// Returns whether this status is terminal (no further transitions).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Canceled | Self::Rejected
        )
    }

    /// Returns the uppercase snake_case string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Submitted => "SUBMITTED",
            Self::Working => "WORKING",
            Self::Completed => "COMPLETED",
            Self::Failed => "FAILED",
            Self::Canceled => "CANCELED",
            Self::Rejected => "REJECTED",
            Self::InputRequired => "INPUT_REQUIRED",
            Self::AuthRequired => "AUTH_REQUIRED",
        }
    }
}

// ===========================================================================
// Content Parts
// ===========================================================================

/// A content part within a Message or Artifact.
///
/// Parts are the atomic content units — text, files, structured data, or
/// inline binary data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Part {
    /// Plain text content.
    Text { text: String },
    /// A file reference (by URL or path).
    File {
        /// URI or file path.
        uri: String,
        /// Optional MIME type hint.
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },
    /// Structured data (JSON-serializable key-value pairs).
    Data {
        /// The structured payload.
        data: serde_json::Value,
    },
    /// Inline binary data (base64-encoded).
    InlineData {
        /// Base64-encoded content.
        data: String,
        /// MIME type of the inline data.
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },
}

// ===========================================================================
// Message
// ===========================================================================

/// Role of a message sender in the A2A protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Agent,
    System,
}

/// An A2A Message containing one or more content parts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// Who sent this message.
    pub role: MessageRole,
    /// Content parts.
    pub parts: Vec<Part>,
    /// Optional context ID to correlate messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Optional metadata (task ID, timestamps, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Message {
    /// Create a new user message with a single text part.
    pub fn user_text(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            parts: vec![Part::Text { text: text.into() }],
            context: None,
            metadata: None,
        }
    }

    /// Create a new agent message with a single text part.
    pub fn agent_text(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Agent,
            parts: vec![Part::Text { text: text.into() }],
            context: None,
            metadata: None,
        }
    }
}

// ===========================================================================
// Artifact
// ===========================================================================

/// An artifact produced by an agent during task execution.
///
/// Artifacts represent outputs like code files, diagrams, documents, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Artifact {
    /// Unique artifact identifier.
    pub id: String,
    /// Human-readable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Description of the artifact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Content parts that make up this artifact.
    pub parts: Vec<Part>,
    /// Creation timestamp (ISO 8601).
    pub created_at: String,
}

// ===========================================================================
// Task
// ===========================================================================

/// An A2A Task — the central unit of work in the protocol.
///
/// A task encapsulates a conversation between a user and one or more agents,
/// tracking messages, artifacts, and status through a defined lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    /// Unique task identifier.
    pub id: String,
    /// Current task status.
    #[serde(default)]
    pub status: TaskStatus,
    /// Optional session ID for multi-turn conversations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Messages exchanged during this task.
    #[serde(default)]
    pub messages: Vec<Message>,
    /// Artifacts produced by this task.
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
    /// Arbitrary metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Task {
    /// Create a new task with the given ID, defaulting to SUBMITTED status.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: TaskStatus::default(),
            session_id: None,
            messages: Vec::new(),
            artifacts: Vec::new(),
            metadata: None,
        }
    }
}

// ===========================================================================
// AgentCard
// ===========================================================================

/// An Agent's self-description card, following the A2A discovery spec.
///
/// Served via `GET /agent-card`, this describes an agent's capabilities,
/// authentication requirements, and supported operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentCard {
    /// Human-readable agent name.
    pub name: String,
    /// Description of what this agent does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The agent endpoint URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// Capabilities this agent supports.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Supported A2A operations (e.g. "sendMessage", "streamMessage").
    #[serde(default)]
    pub supported_operations: Vec<String>,
    /// Authentication requirements.
    #[serde(default)]
    pub auth: AuthInfo,
    /// Protocol version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Authentication information for an agent endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AuthInfo {
    /// Required authentication schemes (e.g. "bearer", "oauth2").
    #[serde(default)]
    pub schemes: Vec<String>,
}

// ===========================================================================
// A2A Error
// ===========================================================================

/// Standard A2A error response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2AError {
    /// Machine-readable error code.
    pub code: i64,
    /// Human-readable error message.
    pub message: String,
    /// Optional additional details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl A2AError {
    /// Create a new error with code and message.
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    /// Standard A2A error: task not found.
    pub fn task_not_found(task_id: &str) -> Self {
        Self::new(-32001, format!("Task not found: {}", task_id))
    }

    /// Standard A2A error: method not found.
    pub fn method_not_found(method: &str) -> Self {
        Self::new(-32601, format!("Method not found: {}", method))
    }

    /// Standard A2A error: invalid params.
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::new(-32602, msg)
    }

    /// Standard A2A error: internal error.
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::new(-32603, msg)
    }
}

impl std::fmt::Display for A2AError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A2AError({}): {}", self.code, self.message)
    }
}

impl std::error::Error for A2AError {}

// ===========================================================================
// JSON-RPC Request / Response types
// ===========================================================================

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    #[serde(default = "jsonrpc_version")]
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
}

fn jsonrpc_version() -> String {
    "2.0".to_string()
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request.
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id: Some(serde_json::Value::Number(1.into())),
        }
    }
}

/// A JSON-RPC 2.0 success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    #[serde(default = "jsonrpc_version")]
    pub jsonrpc: String,
    pub result: T,
    pub id: serde_json::Value,
}

impl<T> JsonRpcResponse<T> {
    /// Create a success response with the given result.
    pub fn ok(result: T, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result,
            id,
        }
    }
}

/// A JSON-RPC 2.0 error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcErrorResponse {
    #[serde(default = "jsonrpc_version")]
    pub jsonrpc: String,
    pub error: A2AError,
    pub id: serde_json::Value,
}

// ===========================================================================
// A2A Method-specific request/response types
// ===========================================================================

/// Request body for `sendMessage`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    /// The task to send the message to.
    pub task: Task,
    /// Optional: hint for stream mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// Response for `sendMessage` (non-streaming).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    /// The updated task after processing the message.
    pub task: Task,
}

/// Request body for `getTask`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskRequest {
    /// The task ID to retrieve.
    pub id: String,
}

/// Response for `getTask`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskResponse {
    pub task: Task,
}

/// Request body for `cancelTask`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskRequest {
    /// The task ID to cancel.
    pub id: String,
}

/// Response for `cancelTask`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskResponse {
    pub task: Task,
}

/// Response for `listTasks`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksResponse {
    pub tasks: Vec<Task>,
}

// ===========================================================================
// Connection-Centric model types (for remote agent support)
// ===========================================================================

/// How an agent connects to its runtime.
///
/// This is the core discrimator for the execution path:
/// - `Local`: spawn a CLI process on this machine (existing behavior).
/// - `Remote`: connect to a remote A2A endpoint via HTTP.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionMode {
    /// Agent runs via a local CLI process.
    #[default]
    Local,
    /// Agent runs on a remote A2A endpoint.
    Remote { connection_id: String },
}

/// Authentication mechanism for a remote connection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    /// No authentication required.
    #[default]
    None,
    /// API key / Bearer token (stored in Keyring).
    ApiKey,
    /// OAuth2 flow (reserved for future use).
    OAuth2,
}

/// Current status of a remote connection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionStatus {
    /// Connection is live and healthy.
    Online,
    /// Connection is not reachable.
    Offline,
    /// Connection encountered an error.
    Error,
    /// Status has not been checked yet.
    #[default]
    Unknown,
}

/// A remote A2A endpoint configuration.
///
/// Represents a single remote server that one or more agents can connect to.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoteConnection {
    /// Unique connection identifier.
    pub id: String,
    /// Human-readable name (e.g. "My Dev Server").
    pub name: String,
    /// The A2A endpoint URL (e.g. "https://dev-server:8443/a2a").
    pub endpoint_url: String,
    /// Authentication mechanism.
    #[serde(default)]
    pub auth_type: AuthType,
    /// Current connection status.
    #[serde(default)]
    pub status: ConnectionStatus,
    /// Cached AgentCard from the remote endpoint (TTL-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_agent_card: Option<AgentCard>,
    /// ISO 8601 timestamp of the last successful health check.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_health_check_at: Option<String>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 last-update timestamp.
    pub updated_at: String,
}

/// Push notification configuration (reserved for Phase 4: multi-agent collaboration).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PushNotificationConfig {
    /// The callback URL for push notifications.
    pub url: String,
    /// Authentication token for the push endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// List of event types to subscribe to (empty = all events).
    #[serde(default)]
    pub events: Vec<String>,
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- TaskStatus tests ---

    #[test]
    fn test_task_status_default() {
        assert_eq!(TaskStatus::default(), TaskStatus::Submitted);
    }

    #[test]
    fn test_task_status_is_terminal() {
        assert!(!TaskStatus::Submitted.is_terminal());
        assert!(!TaskStatus::Working.is_terminal());
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(TaskStatus::Canceled.is_terminal());
        assert!(TaskStatus::Rejected.is_terminal());
    }

    #[test]
    fn test_task_status_serde_roundtrip() {
        let statuses = vec![
            TaskStatus::Submitted,
            TaskStatus::Working,
            TaskStatus::Completed,
            TaskStatus::Failed,
            TaskStatus::Canceled,
            TaskStatus::Rejected,
            TaskStatus::InputRequired,
            TaskStatus::AuthRequired,
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let back: TaskStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn test_task_status_json_format() {
        let json = serde_json::to_string(&TaskStatus::InputRequired).unwrap();
        assert_eq!(json, "\"INPUT_REQUIRED\"");
    }

    // --- Part tests ---

    #[test]
    fn test_text_part_serde() {
        let part = Part::Text {
            text: "hello".into(),
        };
        let json = serde_json::to_string(&part).unwrap();
        let back: Part = serde_json::from_str(&json).unwrap();
        assert_eq!(part, back);
    }

    #[test]
    fn test_file_part_serde() {
        let part = Part::File {
            uri: "file:///tmp/out.txt".into(),
            mime_type: Some("text/plain".into()),
        };
        let json = serde_json::to_string(&part).unwrap();
        let back: Part = serde_json::from_str(&json).unwrap();
        assert_eq!(part, back);
    }

    #[test]
    fn test_data_part_serde() {
        let part = Part::Data {
            data: serde_json::json!({"key": "value"}),
        };
        let json = serde_json::to_string(&part).unwrap();
        let back: Part = serde_json::from_str(&json).unwrap();
        assert_eq!(part, back);
    }

    #[test]
    fn test_inline_data_part_serde() {
        let part = Part::InlineData {
            data: "SGVsbG8=".into(),
            mime_type: Some("text/plain".into()),
        };
        let json = serde_json::to_string(&part).unwrap();
        let back: Part = serde_json::from_str(&json).unwrap();
        assert_eq!(part, back);
    }

    // --- Message tests ---

    #[test]
    fn test_message_user_text() {
        let msg = Message::user_text("Hello agent");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.parts.len(), 1);
        assert_eq!(msg.context, None);
    }

    #[test]
    fn test_message_agent_text() {
        let msg = Message::agent_text("Hello user");
        assert_eq!(msg.role, MessageRole::Agent);
    }

    #[test]
    fn test_message_serde_roundtrip() {
        let msg = Message {
            role: MessageRole::Agent,
            parts: vec![
                Part::Text {
                    text: "result".into(),
                },
                Part::Data {
                    data: serde_json::json!({"count": 42}),
                },
            ],
            context: Some("ctx-123".into()),
            metadata: Some(serde_json::json!({"timestamp": "2026-01-01"})),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    // --- Task tests ---

    #[test]
    fn test_task_new() {
        let task = Task::new("task-001");
        assert_eq!(task.id, "task-001");
        assert_eq!(task.status, TaskStatus::Submitted);
        assert!(task.messages.is_empty());
        assert!(task.artifacts.is_empty());
        assert!(task.session_id.is_none());
    }

    #[test]
    fn test_task_serde_roundtrip() {
        let task = Task {
            id: "task-002".into(),
            status: TaskStatus::Working,
            session_id: Some("sess-abc".into()),
            messages: vec![Message::user_text("do something")],
            artifacts: vec![Artifact {
                id: "art-1".into(),
                name: Some("output.txt".into()),
                description: None,
                parts: vec![Part::Text {
                    text: "content".into(),
                }],
                created_at: "2026-04-16T00:00:00Z".into(),
            }],
            metadata: Some(serde_json::json!({"priority": "high"})),
        };
        let json = serde_json::to_string(&task).unwrap();
        let back: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(task, back);
    }

    // --- AgentCard tests ---

    #[test]
    fn test_agent_card_serde() {
        let card = AgentCard {
            name: "Claude Code".into(),
            description: Some("AI coding assistant".into()),
            endpoint: Some("http://localhost:8080/a2a".into()),
            capabilities: vec!["streaming".into(), "tool_use".into()],
            supported_operations: vec![
                "sendMessage".into(),
                "streamMessage".into(),
                "getTask".into(),
            ],
            auth: AuthInfo {
                schemes: vec![],
            },
            version: Some("1.0.0".into()),
        };
        let json = serde_json::to_string(&card).unwrap();
        let back: AgentCard = serde_json::from_str(&json).unwrap();
        assert_eq!(card, back);
    }

    // --- A2AError tests ---

    #[test]
    fn test_a2a_error_constructors() {
        let e = A2AError::task_not_found("t-1");
        assert_eq!(e.code, -32001);
        assert!(e.message.contains("t-1"));

        let e = A2AError::method_not_found("foo");
        assert_eq!(e.code, -32601);

        let e = A2AError::invalid_params("bad");
        assert_eq!(e.code, -32602);

        let e = A2AError::internal_error("oops");
        assert_eq!(e.code, -32603);
    }

    // --- JSON-RPC tests ---

    #[test]
    fn test_json_rpc_request() {
        let req = JsonRpcRequest::new(
            "sendMessage",
            Some(serde_json::json!({"task": {"id": "t-1"}})),
        );
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "sendMessage");
        assert!(req.params.is_some());
    }

    #[test]
    fn test_json_rpc_request_serde() {
        let req = JsonRpcRequest::new("getTask", Some(serde_json::json!({"id": "t-1"})));
        let json = serde_json::to_string(&req).unwrap();
        let back: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.method, back.method);
    }

    // --- Connection model tests ---

    #[test]
    fn test_connection_mode_default() {
        assert_eq!(ConnectionMode::default(), ConnectionMode::Local);
    }

    #[test]
    fn test_connection_mode_local_serde() {
        let mode = ConnectionMode::Local;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"local\"");
        let back: ConnectionMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, back);
    }

    #[test]
    fn test_connection_mode_remote_serde() {
        let mode = ConnectionMode::Remote {
            connection_id: "conn-1".into(),
        };
        let json = serde_json::to_string(&mode).unwrap();
        let back: ConnectionMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, back);
    }

    #[test]
    fn test_connection_mode_remote_json_structure() {
        let mode = ConnectionMode::Remote {
            connection_id: "conn-1".into(),
        };
        let val = serde_json::to_value(&mode).unwrap();
        // Should serialize as {"remote": {"connection_id": "conn-1"}}
        let obj = val.as_object().unwrap();
        assert!(obj.contains_key("remote"));
        let inner = obj.get("remote").unwrap().as_object().unwrap();
        assert_eq!(inner.get("connection_id").unwrap().as_str(), Some("conn-1"));
    }

    #[test]
    fn test_auth_type_default() {
        assert_eq!(AuthType::default(), AuthType::None);
    }

    #[test]
    fn test_auth_type_serde_roundtrip() {
        let types = vec![AuthType::None, AuthType::ApiKey, AuthType::OAuth2];
        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let back: AuthType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn test_connection_status_default() {
        assert_eq!(ConnectionStatus::default(), ConnectionStatus::Unknown);
    }

    #[test]
    fn test_connection_status_serde_roundtrip() {
        let statuses = vec![
            ConnectionStatus::Online,
            ConnectionStatus::Offline,
            ConnectionStatus::Error,
            ConnectionStatus::Unknown,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let back: ConnectionStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn test_connection_status_kebab_case() {
        let json = serde_json::to_string(&ConnectionStatus::Online).unwrap();
        assert_eq!(json, "\"online\"");
        // The spec says kebab-case for this enum, but the values are single words.
    }

    #[test]
    fn test_remote_connection_serde_roundtrip() {
        let conn = RemoteConnection {
            id: "conn-1".into(),
            name: "My Dev Server".into(),
            endpoint_url: "https://dev-server:8443/a2a".into(),
            auth_type: AuthType::ApiKey,
            status: ConnectionStatus::Online,
            cached_agent_card: None,
            last_health_check_at: Some("2026-04-16T10:00:00Z".into()),
            created_at: "2026-04-14T00:00:00Z".into(),
            updated_at: "2026-04-16T10:00:00Z".into(),
        };
        let json = serde_json::to_string(&conn).unwrap();
        let back: RemoteConnection = serde_json::from_str(&json).unwrap();
        assert_eq!(conn, back);
    }

    #[test]
    fn test_push_notification_config_serde() {
        let config = PushNotificationConfig {
            url: "https://callback.example.com/push".into(),
            token: Some("secret-token".into()),
            events: vec!["task.completed".into(), "task.failed".into()],
        };
        let json = serde_json::to_string(&config).unwrap();
        let back: PushNotificationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, back);
    }

    #[test]
    fn test_send_message_request_serde() {
        let req = SendMessageRequest {
            task: Task::new("t-1"),
            stream: Some(true),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: SendMessageRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.task.id, back.task.id);
        assert_eq!(req.stream, back.stream);
    }
}
