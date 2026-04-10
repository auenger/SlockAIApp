//! Storage module.
//!
//! Handles JSONL conversation log persistence,
//! SQLite database for structured metadata,
//! secure API key management via OS Keyring,
//! and activity log persistence (dual-write: JSONL + SQLite).

pub mod activity;
pub mod db;
pub mod db_helpers;
pub mod jsonl;
pub mod keyring;
