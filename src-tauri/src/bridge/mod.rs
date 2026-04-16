//! Bridge module — Remote Workspace Gateway.
//!
//! Provides a standalone `az-bridge` binary that exposes a local workspace
//! as an A2A endpoint with extended bridge.* protocol for remote management.

pub mod config;
pub mod handlers;
pub mod server;

pub use config::BridgeConfig;
pub use server::BridgeServer;
