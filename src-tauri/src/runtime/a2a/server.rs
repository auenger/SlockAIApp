//! A2A Server skeleton.
//!
//! Provides a configurable HTTP server that exposes A2A protocol endpoints.
//! Handlers are registered via closures, allowing the server to be used
//! both for testing and as a real A2A endpoint adapter.

use super::types::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Handler function type for processing incoming JSON-RPC requests.
type RpcHandler = Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value, A2AError> + Send + Sync>;

/// Configuration for an A2A server instance.
#[derive(Debug, Clone)]
pub struct A2AServerConfig {
    /// Address to bind to (e.g. "0.0.0.0").
    pub host: String,
    /// Port to listen on.
    pub port: u16,
    /// Agent card to serve at `GET /agent-card`.
    pub agent_card: AgentCard,
}

impl A2AServerConfig {
    /// Create a default config listening on 0.0.0.0:8080 (LAN-accessible).
    pub fn new(agent_card: AgentCard) -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            agent_card,
        }
    }

    /// Set a custom host.
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Set a custom port.
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

/// A2A Server: registers handlers for JSON-RPC methods and serves them over HTTP.
///
/// This is a skeleton implementation that provides:
/// - Route registration for A2A JSON-RPC methods
/// - GET /agent-card endpoint
/// - POST / for JSON-RPC dispatch
///
/// The actual server loop is not started here — this struct is designed to
/// be integrated into a larger HTTP server (e.g. via hyper or axum) or used
/// for testing with mock responses.
pub struct A2AServer {
    config: A2AServerConfig,
    handlers: Arc<Mutex<HashMap<String, RpcHandler>>>,
}

impl A2AServer {
    /// Create a new A2A server with the given configuration.
    pub fn new(config: A2AServerConfig) -> Self {
        Self {
            config,
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the server configuration.
    pub fn config(&self) -> &A2AServerConfig {
        &self.config
    }

    /// Register a handler for a JSON-RPC method.
    pub fn register_handler<F>(&self, method: &str, handler: F)
    where
        F: Fn(serde_json::Value) -> Result<serde_json::Value, A2AError> + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.lock().expect("handler lock poisoned");
        handlers.insert(method.to_string(), Box::new(handler));
    }

    /// Dispatch an incoming JSON-RPC request to the appropriate handler.
    pub fn dispatch(&self, request: &JsonRpcRequest) -> Result<serde_json::Value, A2AError> {
        let handlers = self.handlers.lock().expect("handler lock poisoned");
        let handler = handlers
            .get(&request.method)
            .ok_or_else(|| A2AError::method_not_found(&request.method))?;

        let params = request.params.clone().unwrap_or_default();
        handler(params)
    }

    /// Get the agent card for this server.
    pub fn agent_card(&self) -> &AgentCard {
        &self.config.agent_card
    }

    /// Build the JSON-RPC response for a successful dispatch.
    pub fn handle_rpc_request(
        &self,
        body: &str,
    ) -> String {
        // Parse the incoming JSON-RPC request
        let rpc_req: serde_json::Value = match serde_json::from_str(body) {
            Ok(v) => v,
            Err(e) => {
                let err_resp = JsonRpcErrorResponse {
                    jsonrpc: "2.0".to_string(),
                    error: A2AError::new(-32700, format!("Parse error: {}", e)),
                    id: serde_json::Value::Null,
                };
                return serde_json::to_string(&err_resp).unwrap_or_default();
            }
        };

        let id = rpc_req.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = rpc_req
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("");

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: rpc_req.get("params").cloned(),
            id: Some(id.clone()),
        };

        match self.dispatch(&request) {
            Ok(result) => {
                let resp = JsonRpcResponse::ok(result, id);
                serde_json::to_string(&resp).unwrap_or_default()
            }
            Err(error) => {
                let err_resp = JsonRpcErrorResponse {
                    jsonrpc: "2.0".to_string(),
                    error,
                    id,
                };
                serde_json::to_string(&err_resp).unwrap_or_default()
            }
        }
    }

    /// Register default A2A method handlers with in-memory task storage.
    ///
    /// This sets up basic handlers for:
    /// - sendMessage
    /// - getTask
    /// - cancelTask
    /// - listTasks
    pub fn register_default_handlers(&self) {
        let tasks: Arc<Mutex<HashMap<String, Task>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // sendMessage handler
        let tasks_clone = tasks.clone();
        self.register_handler("sendMessage", move |params| {
            let req: SendMessageRequest = serde_json::from_value(params)
                .map_err(|e| A2AError::invalid_params(format!("Invalid sendMessage params: {}", e)))?;
            let mut tasks = tasks_clone.lock().unwrap();
            let task = tasks
                .entry(req.task.id.clone())
                .or_insert_with(|| req.task);
            task.status = TaskStatus::Working;
            let result = serde_json::to_value(SendMessageResponse {
                task: task.clone(),
            })
            .unwrap_or_default();
            Ok(result)
        });

        // getTask handler
        let tasks_clone = tasks.clone();
        self.register_handler("getTask", move |params| {
            let req: GetTaskRequest = serde_json::from_value(params)
                .map_err(|e| A2AError::invalid_params(format!("Invalid getTask params: {}", e)))?;
            let tasks = tasks_clone.lock().unwrap();
            let task = tasks
                .get(&req.id)
                .ok_or_else(|| A2AError::task_not_found(&req.id))?
                .clone();
            let result = serde_json::to_value(GetTaskResponse { task }).unwrap_or_default();
            Ok(result)
        });

        // cancelTask handler
        let tasks_clone = tasks.clone();
        self.register_handler("cancelTask", move |params| {
            let req: CancelTaskRequest = serde_json::from_value(params)
                .map_err(|e| A2AError::invalid_params(format!("Invalid cancelTask params: {}", e)))?;
            let mut tasks = tasks_clone.lock().unwrap();
            let task = tasks
                .get_mut(&req.id)
                .ok_or_else(|| A2AError::task_not_found(&req.id))?;
            task.status = TaskStatus::Canceled;
            let result = serde_json::to_value(CancelTaskResponse {
                task: task.clone(),
            })
            .unwrap_or_default();
            Ok(result)
        });

        // listTasks handler
        let tasks_clone = tasks;
        self.register_handler("listTasks", move |_| {
            let tasks = tasks_clone.lock().unwrap();
            let result =
                serde_json::to_value(ListTasksResponse {
                    tasks: tasks.values().cloned().collect(),
                })
                .unwrap_or_default();
            Ok(result)
        });
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_agent_card() -> AgentCard {
        AgentCard {
            name: "TestAgent".into(),
            description: Some("Test agent".into()),
            endpoint: Some("http://localhost:8080/a2a".into()),
            capabilities: vec!["streaming".into()],
            supported_operations: vec!["sendMessage".into(), "getTask".into()],
            auth: AuthInfo { schemes: vec![] },
            version: Some("1.0.0".into()),
        }
    }

    #[test]
    fn test_server_config_default() {
        let config = A2AServerConfig::new(test_agent_card());
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_server_config_custom() {
        let config = A2AServerConfig::new(test_agent_card())
            .with_host("0.0.0.0")
            .with_port(9090);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 9090);
    }

    #[test]
    fn test_server_dispatch_unknown_method() {
        let server = A2AServer::new(A2AServerConfig::new(test_agent_card()));
        let req = JsonRpcRequest::new("nonexistent", None);
        let result = server.dispatch(&req);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, -32601);
    }

    #[test]
    fn test_server_register_handler() {
        let server = A2AServer::new(A2AServerConfig::new(test_agent_card()));
        server.register_handler("testMethod", |_params| {
            Ok(serde_json::json!({"result": "ok"}))
        });

        let req = JsonRpcRequest::new("testMethod", Some(serde_json::Value::Null));
        let result = server.dispatch(&req).unwrap();
        assert_eq!(result["result"], "ok");
    }

    #[test]
    fn test_server_handle_rpc_request_parse_error() {
        let server = A2AServer::new(A2AServerConfig::new(test_agent_card()));
        let response = server.handle_rpc_request("invalid json{{{");
        assert!(response.contains("-32700"));
    }

    #[test]
    fn test_server_default_handlers_send_message() {
        let server = A2AServer::new(A2AServerConfig::new(test_agent_card()));
        server.register_default_handlers();

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "sendMessage",
            "params": {
                "task": {
                    "id": "t-1",
                    "status": "SUBMITTED",
                    "messages": [],
                    "artifacts": []
                }
            },
            "id": 1
        })
        .to_string();

        let response = server.handle_rpc_request(&body);
        let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["result"]["task"]["status"], "WORKING");
    }

    #[test]
    fn test_server_default_handlers_get_task() {
        let server = A2AServer::new(A2AServerConfig::new(test_agent_card()));
        server.register_default_handlers();

        // First send a message to create the task
        let send_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "sendMessage",
            "params": {
                "task": {
                    "id": "t-2",
                    "status": "SUBMITTED",
                    "messages": [],
                    "artifacts": []
                }
            },
            "id": 1
        })
        .to_string();
        server.handle_rpc_request(&send_body);

        // Now get the task
        let get_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getTask",
            "params": {"id": "t-2"},
            "id": 2
        })
        .to_string();
        let response = server.handle_rpc_request(&get_body);
        let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["result"]["task"]["id"], "t-2");
    }

    #[test]
    fn test_server_default_handlers_cancel_task() {
        let server = A2AServer::new(A2AServerConfig::new(test_agent_card()));
        server.register_default_handlers();

        // Create task
        let send_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "sendMessage",
            "params": {
                "task": {
                    "id": "t-3",
                    "status": "SUBMITTED",
                    "messages": [],
                    "artifacts": []
                }
            },
            "id": 1
        })
        .to_string();
        server.handle_rpc_request(&send_body);

        // Cancel task
        let cancel_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "cancelTask",
            "params": {"id": "t-3"},
            "id": 2
        })
        .to_string();
        let response = server.handle_rpc_request(&cancel_body);
        let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["result"]["task"]["status"], "CANCELED");
    }

    #[test]
    fn test_server_default_handlers_list_tasks() {
        let server = A2AServer::new(A2AServerConfig::new(test_agent_card()));
        server.register_default_handlers();

        // Create two tasks
        for i in 0..2 {
            let send_body = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "sendMessage",
                "params": {
                    "task": {
                        "id": format!("t-l{}", i),
                        "status": "SUBMITTED",
                        "messages": [],
                        "artifacts": []
                    }
                },
                "id": i
            })
            .to_string();
            server.handle_rpc_request(&send_body);
        }

        // List tasks
        let list_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "listTasks",
            "params": {},
            "id": 99
        })
        .to_string();
        let response = server.handle_rpc_request(&list_body);
        let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["result"]["tasks"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_agent_card_endpoint() {
        let server = A2AServer::new(A2AServerConfig::new(test_agent_card()));
        let card = server.agent_card();
        assert_eq!(card.name, "TestAgent");
    }
}
