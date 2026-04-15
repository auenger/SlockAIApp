//! A2A (Agent-to-Agent) protocol module.
//!
//! This module provides the complete A2A protocol implementation including:
//! - Type definitions (`types`) — Task, Message, Artifact, AgentCard, etc.
//! - Transport abstraction (`transport`) — A2ATransport trait + HTTP client.
//! - SSE streaming (`streaming`) — Server-Sent Events support.
//! - Server skeleton (`server`) — A2A HTTP server with handler registration.
//! - Bridge conversion (`bridge`) — StreamEvent <-> A2A Message conversion.
//! - CLI adapters (`adapter`) — Wrap CLI runtimes as A2A endpoints.

pub mod adapter;
pub mod bridge;
pub mod server;
pub mod streaming;
pub mod transport;
pub mod types;

// Re-export the most commonly used types at the module level for convenience.
pub use types::{
    A2AError, AgentCard, Artifact, AuthInfo, AuthType, CancelTaskRequest, CancelTaskResponse,
    ConnectionMode, ConnectionStatus, GetTaskRequest, GetTaskResponse, JsonRpcErrorResponse,
    JsonRpcRequest, JsonRpcResponse, ListTasksResponse, Message, MessageRole, Part,
    PushNotificationConfig, RemoteConnection, SendMessageRequest, SendMessageResponse, Task,
    TaskStatus,
};
pub use transport::{A2AHttpClient, A2ATransport, A2AResult};
pub use adapter::{CliA2AAdapter, AdapterConfig, AdapterState, ClaudeCodeAdapter, CodexAdapter, AdapterServer, ConnectionPool, ListenerConfig, PooledConnection, SocketGuard, generate_agent_card};
