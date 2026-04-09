//! Storage module.
//!
//! Handles JSONL conversation log persistence,
//! Markdown document read/write operations,
//! and secure API key management via OS Keyring.

pub mod jsonl;
pub mod keyring;
