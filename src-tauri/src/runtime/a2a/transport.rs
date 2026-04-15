//! A2A Transport trait and HTTP implementation.
//!
//! Defines the `A2ATransport` trait that abstracts the communication
//! mechanism for A2A protocol operations. Provides an HTTP-based
//! implementation using reqwest for standard JSON-RPC over HTTP.

use super::types::*;
use crate::runtime::StreamEvent;
use std::collections::HashMap;

/// Result type alias for A2A transport operations.
pub type A2AResult<T> = Result<T, A2AError>;

/// Transport trait for A2A protocol communication.
///
/// Abstracts over the underlying transport (HTTP, gRPC, local, etc.)
/// so that higher-level code can be transport-agnostic.
pub trait A2ATransport: Send + Sync {
    /// Send a message to a task (non-streaming).
    fn send_message(
        &self,
        task: Task,
        message: Message,
    ) -> A2AResult<SendMessageResponse>;

    /// Stream messages to/from a task (SSE-based).
    /// Returns a receiver that yields streaming events compatible with
    /// the existing runtime StreamEvent model.
    fn stream_message(
        &self,
        task_id: &str,
        message: Message,
    ) -> A2AResult<std::sync::mpsc::Receiver<StreamEvent>>;

    /// Get the current state of a task.
    fn get_task(&self, task_id: &str) -> A2AResult<Task>;

    /// Cancel a running task.
    fn cancel_task(&self, task_id: &str) -> A2AResult<Task>;

    /// List all tasks known to the remote agent.
    fn list_tasks(&self) -> A2AResult<Vec<Task>>;

    /// Retrieve the agent's self-description card.
    fn get_agent_card(&self) -> A2AResult<AgentCard>;
}

// ===========================================================================
// HTTP Transport implementation
// ===========================================================================

/// HTTP-based A2A transport client using reqwest.
///
/// Communicates with a remote A2A-compatible endpoint via JSON-RPC 2.0
/// over HTTP POST. Supports both one-shot and streaming (SSE) modes.
pub struct A2AHttpClient {
    /// Base URL of the remote A2A endpoint (e.g. "http://localhost:8080/a2a").
    base_url: String,
    /// HTTP client instance.
    client: reqwest::blocking::Client,
    /// Optional authentication headers.
    auth_headers: HashMap<String, String>,
}

impl A2AHttpClient {
    /// Create a new HTTP client pointing to the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("failed to build reqwest client"),
            auth_headers: HashMap::new(),
        }
    }

    /// Create a client with a Bearer token for authentication.
    pub fn with_bearer_token(base_url: impl Into<String>, token: &str) -> Self {
        let mut auth_headers = HashMap::new();
        auth_headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", token),
        );
        Self {
            base_url: base_url.into(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("failed to build reqwest client"),
            auth_headers,
        }
    }

    /// Create a client with custom auth headers.
    pub fn with_auth_headers(
        base_url: impl Into<String>,
        auth_headers: HashMap<String, String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("failed to build reqwest client"),
            auth_headers,
        }
    }

    /// Build a JSON-RPC request with the given method and params.
    fn build_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> JsonRpcRequest {
        JsonRpcRequest::new(method, Some(params))
    }

    /// Send a JSON-RPC request and parse the response.
    fn send_rpc<T: serde::de::DeserializeOwned>(
        &self,
        rpc_request: &JsonRpcRequest,
    ) -> A2AResult<T> {
        let url = format!("{}/", self.base_url.trim_end_matches('/'));

        let mut builder = self.client.post(&url);
        for (key, value) in &self.auth_headers {
            builder = builder.header(key.as_str(), value.as_str());
        }

        let response = builder
            .json(rpc_request)
            .send()
            .map_err(|e| A2AError::internal_error(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(A2AError::new(
                status.as_u16() as i64,
                format!("HTTP error {}: {}", status, body),
            ));
        }

        // Read response body as owned String to avoid lifetime issues with serde
        let body_text = response
            .text()
            .map_err(|e| A2AError::internal_error(format!("Failed to read response: {}", e)))?;

        // Try to parse as a generic JSON value first, then determine success vs error
        let json_value: serde_json::Value = serde_json::from_str(&body_text)
            .map_err(|e| {
                A2AError::internal_error(format!(
                    "Failed to parse response JSON: {} — body: {}",
                    e,
                    &body_text[..body_text.len().min(200)]
                ))
            })?;

        // Check if it's a JSON-RPC error response (has "error" field)
        if let Some(error_obj) = json_value.get("error") {
            let error: A2AError = serde_json::from_value(error_obj.clone())
                .unwrap_or_else(|_| A2AError::internal_error("Unknown error format"));
            return Err(error);
        }

        // Parse as success response — extract the "result" field
        let result_value = json_value.get("result")
            .ok_or_else(|| A2AError::internal_error("Missing 'result' field in response"))?
            .clone();

        let result: T = serde_json::from_value(result_value)
            .map_err(|e| {
                A2AError::internal_error(format!("Failed to deserialize result: {}", e))
            })?;

        Ok(result)
    }
}

impl A2ATransport for A2AHttpClient {
    fn send_message(
        &self,
        task: Task,
        message: Message,
    ) -> A2AResult<SendMessageResponse> {
        let mut task_with_msg = task;
        task_with_msg.messages.push(message);

        let params = serde_json::to_value(SendMessageRequest {
            task: task_with_msg,
            stream: Some(false),
        })
        .unwrap_or_default();

        let rpc = self.build_request("sendMessage", params);
        self.send_rpc(&rpc)
    }

    fn stream_message(
        &self,
        task_id: &str,
        message: Message,
    ) -> A2AResult<std::sync::mpsc::Receiver<StreamEvent>> {
        // SSE streaming is handled by the streaming module
        // Clone the client to move it into the background thread
        use super::streaming::open_sse_stream;

        let stream_client = self.client.clone();
        let url = format!("{}/tasks/{}/messages", self.base_url.trim_end_matches('/'), task_id);
        open_sse_stream(stream_client, &url, message, &self.auth_headers)
    }

    fn get_task(&self, task_id: &str) -> A2AResult<Task> {
        let params = serde_json::to_value(GetTaskRequest {
            id: task_id.to_string(),
        })
        .unwrap_or_default();

        let rpc = self.build_request("getTask", params);
        let resp: GetTaskResponse = self.send_rpc(&rpc)?;
        Ok(resp.task)
    }

    fn cancel_task(&self, task_id: &str) -> A2AResult<Task> {
        let params = serde_json::to_value(CancelTaskRequest {
            id: task_id.to_string(),
        })
        .unwrap_or_default();

        let rpc = self.build_request("cancelTask", params);
        let resp: CancelTaskResponse = self.send_rpc(&rpc)?;
        Ok(resp.task)
    }

    fn list_tasks(&self) -> A2AResult<Vec<Task>> {
        let rpc = self.build_request("listTasks", serde_json::Value::Object(Default::default()));
        let resp: ListTasksResponse = self.send_rpc(&rpc)?;
        Ok(resp.tasks)
    }

    fn get_agent_card(&self) -> A2AResult<AgentCard> {
        let url = format!(
            "{}/agent-card",
            self.base_url.trim_end_matches('/')
        );

        let mut builder = self.client.get(&url);
        for (key, value) in &self.auth_headers {
            builder = builder.header(key.as_str(), value.as_str());
        }

        let response = builder
            .send()
            .map_err(|e| A2AError::internal_error(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            return Err(A2AError::new(
                status.as_u16() as i64,
                format!("HTTP error getting agent card: {}", status),
            ));
        }

        response
            .json::<AgentCard>()
            .map_err(|e| A2AError::internal_error(format!("Failed to parse agent card: {}", e)))
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a2a_http_client_new() {
        let client = A2AHttpClient::new("http://localhost:8080/a2a");
        assert_eq!(client.base_url, "http://localhost:8080/a2a");
        assert!(client.auth_headers.is_empty());
    }

    #[test]
    fn test_a2a_http_client_with_bearer_token() {
        let client = A2AHttpClient::with_bearer_token("http://localhost:8080/a2a", "my-token");
        assert!(client.auth_headers.contains_key("Authorization"));
        assert_eq!(
            client.auth_headers.get("Authorization").unwrap(),
            "Bearer my-token"
        );
    }

    #[test]
    fn test_build_request() {
        let client = A2AHttpClient::new("http://localhost:8080/a2a");
        let rpc = client.build_request("getTask", serde_json::json!({"id": "t-1"}));
        assert_eq!(rpc.method, "getTask");
        assert_eq!(rpc.jsonrpc, "2.0");
        assert!(rpc.params.is_some());
    }

    #[test]
    fn test_a2a_error_from_http_status() {
        let err = A2AError::new(404, "Not found".to_string());
        assert_eq!(err.code, 404);
    }
}
