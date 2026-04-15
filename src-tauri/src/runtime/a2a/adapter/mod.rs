//! A2A Server Adapter module.
//!
//! Provides adapters that wrap existing CLI-based runtimes (Claude Code, Codex)
//! into A2A Server endpoints. This enables local agents to be accessed via the
//! standard A2A protocol, either through Unix sockets (local) or HTTP (remote).
//!
//! Architecture:
//! ```text
//! A2A Server (HTTP/Unix Socket)
//!   -> A2A Handler (Task CRUD + SendMessage)
//!     -> CliA2AAdapter (trait)
//!       -> ClaudeCodeAdapter | CodexAdapter
//!         -> CLI execute() (existing runtime code)
//! ```

pub mod claude_adapter;
pub mod cli_adapter;
pub mod codex_adapter;
pub mod handler;

pub use cli_adapter::{AdapterConfig, AdapterState, CliA2AAdapter};
pub use claude_adapter::ClaudeCodeAdapter;
pub use codex_adapter::CodexAdapter;
pub use handler::{AdapterServer, ConnectionPool, ListenerConfig, PooledConnection, SocketGuard, generate_agent_card, handle_tcp_connection, start_tcp_listener};
