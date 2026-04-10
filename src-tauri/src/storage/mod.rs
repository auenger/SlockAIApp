//! Storage module.
//!
//! Handles JSONL conversation log persistence,
//! Markdown document read/write operations,
//! secure API key management via OS Keyring,
//! and activity log persistence.

pub mod activity;
pub mod jsonl;
pub mod keyring;
