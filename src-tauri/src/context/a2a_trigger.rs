//! Agent-to-Agent (A2A) trigger chain execution engine.
//!
//! When an Agent produces a response that contains @{agent} mentions of other
//! Channel members, this module manages the recursive execution chain:
//!
//! 1. Parse the Agent's response for @mentions of other Channel members.
//! 2. Apply safety checks (depth limit, deduplication).
//! 3. Recursively trigger each mentioned Agent with proper context.
//!
//! ## Safety Mechanisms
//!
//! - **Max depth**: Trigger chains are limited to `DEFAULT_MAX_DEPTH` (3) levels.
//!   A → B → C → STOP.  Prevents runaway chains.
//! - **Deduplication**: An Agent that has already been triggered in the current
//!   chain is never triggered again.  Prevents infinite loops (A → B → A → ...).
//! - **Timeout**: Each Agent execution has an independent timeout.
//!
//! ## Events
//!
//! The engine emits Tauri events so the frontend can visualize the A2A chain:
//! - `agent://channel-a2a-start`: Emitted when an A2A trigger fires.
//! - `agent://channel-a2a-depth-exceeded`: Emitted when the depth limit is hit.

use std::collections::HashSet;
use std::sync::mpsc::Receiver;

use tauri::Emitter;

use crate::workspace::mention;

/// Default maximum trigger chain depth.
pub const DEFAULT_MAX_DEPTH: u32 = 3;

// ===========================================================================
// Trigger Context
// ===========================================================================

/// Tracks the state of an A2A trigger chain.
///
/// Carries the current depth, the maximum allowed depth, and the set of
/// Agent IDs that have already been triggered in this chain (for dedup).
#[derive(Debug, Clone)]
pub struct TriggerContext {
    /// Current depth in the trigger chain (0 = initial user-triggered execution).
    pub depth: u32,
    /// Maximum allowed depth before the chain is terminated.
    pub max_depth: u32,
    /// Agent IDs that have already been triggered in this chain.
    pub triggered_agents: HashSet<String>,
}

impl TriggerContext {
    /// Create a fresh trigger context for a new user-initiated message.
    ///
    /// The first Agent execution happens at depth 0.  A2A triggers start at depth 1.
    pub fn new() -> Self {
        Self {
            depth: 0,
            max_depth: DEFAULT_MAX_DEPTH,
            triggered_agents: HashSet::new(),
        }
    }

    /// Create a trigger context with a custom max depth.
    pub fn with_max_depth(max_depth: u32) -> Self {
        Self {
            depth: 0,
            max_depth,
            triggered_agents: HashSet::new(),
        }
    }

    /// Check whether a given Agent ID can still be triggered.
    ///
    /// Returns `false` if the Agent has already been triggered in this chain
    /// or if the maximum depth has been reached.
    pub fn can_trigger(&self, agent_id: &str) -> bool {
        !self.triggered_agents.contains(agent_id) && self.depth < self.max_depth
    }

    /// Create a child context for a triggered Agent.
    ///
    /// Increments depth and records the triggered Agent.
    pub fn child_for(&self, agent_id: &str) -> Self {
        let mut child = self.clone();
        child.depth += 1;
        child.triggered_agents.insert(agent_id.to_string());
        child
    }
}

impl Default for TriggerContext {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// A2A extraction
// ===========================================================================

/// Extract valid A2A trigger targets from an Agent's response text.
///
/// Returns a list of Agent IDs that:
/// 1. Are @mentioned in the response.
/// 2. Are members of the Channel.
/// 3. Have not already been triggered in this chain (dedup).
/// 4. Would not exceed the max depth limit.
pub fn extract_valid_triggers(
    response: &str,
    channel_members: &[crate::workspace::channel::ChannelMember],
    trigger_ctx: &TriggerContext,
) -> Vec<String> {
    let mentioned = mention::extract_agent_triggers(response, channel_members);

    mentioned
        .into_iter()
        .filter(|id| trigger_ctx.can_trigger(id))
        .collect()
}

// ===========================================================================
// A2A execution result
// ===========================================================================

/// The outcome of processing A2A triggers for a single Agent's response.
#[derive(Debug)]
pub struct A2aChainResult {
    /// Agent IDs that were successfully triggered.
    pub triggered: Vec<String>,
    /// Agent IDs that were skipped because the depth limit was reached.
    pub depth_exceeded: Vec<String>,
    /// Agent IDs that were skipped due to deduplication.
    pub dedup_skipped: Vec<String>,
}

impl A2aChainResult {
    pub fn new() -> Self {
        Self {
            triggered: Vec::new(),
            depth_exceeded: Vec::new(),
            dedup_skipped: Vec::new(),
        }
    }
}

/// Analyze an Agent's response and classify which mentioned Agents can be
/// triggered vs. which are blocked by safety limits.
///
/// This does **not** execute any triggers; it only classifies them.
pub fn classify_triggers(
    response: &str,
    channel_members: &[crate::workspace::channel::ChannelMember],
    trigger_ctx: &TriggerContext,
) -> A2aChainResult {
    let mentioned = mention::extract_agent_triggers(response, channel_members);
    let mut result = A2aChainResult::new();

    for agent_id in mentioned {
        if trigger_ctx.triggered_agents.contains(&agent_id) {
            result.dedup_skipped.push(agent_id);
        } else if trigger_ctx.depth >= trigger_ctx.max_depth {
            result.depth_exceeded.push(agent_id);
        } else {
            result.triggered.push(agent_id);
        }
    }

    result
}

// ===========================================================================
// Stream processing helper
// ===========================================================================

/// Process a streaming response from an Agent runtime, collecting the full
/// text and forwarding events to the frontend.
///
/// Returns the full collected response text and whether there was an error.
pub fn collect_stream_response(
    receiver: &Receiver<crate::runtime::StreamEvent>,
    app: &tauri::AppHandle,
    channel_id: &str,
    agent_id: &str,
    agent_idx: usize,
    total_agents: usize,
) -> (String, bool) {
    let mut full_response = String::new();
    let mut had_error = false;

    while let Ok(event) = receiver.recv_timeout(std::time::Duration::from_secs(300)) {
        if event.msg_type.as_deref() == Some("assistant") && !event.text.is_empty() {
            full_response.push_str(&event.text);
        }

        // Forward streaming event to frontend
        let _ = app.emit(
            "agent://channel-chunk",
            serde_json::json!({
                "channel_id": channel_id,
                "agent_id": agent_id,
                "agent_index": agent_idx,
                "total_agents": total_agents,
                "event": event,
            }),
        );

        if event.is_done {
            if event.error.is_some() {
                had_error = true;
            }
            break;
        }
    }

    (full_response, had_error)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::channel::ChannelMember;

    fn make_members(ids: &[&str]) -> Vec<ChannelMember> {
        ids.iter()
            .map(|&id| ChannelMember {
                agent_id: id.to_string(),
                role: "member".to_string(),
                joined_at: "2026-01-01T00:00:00Z".to_string(),
            })
            .collect()
    }

    // ---- TriggerContext tests ----

    #[test]
    fn test_trigger_context_new() {
        let ctx = TriggerContext::new();
        assert_eq!(ctx.depth, 0);
        assert_eq!(ctx.max_depth, DEFAULT_MAX_DEPTH);
        assert!(ctx.triggered_agents.is_empty());
    }

    #[test]
    fn test_can_trigger_fresh() {
        let ctx = TriggerContext::new();
        assert!(ctx.can_trigger("codex"));
    }

    #[test]
    fn test_can_trigger_already_triggered() {
        let mut ctx = TriggerContext::new();
        ctx.triggered_agents.insert("codex".to_string());
        assert!(!ctx.can_trigger("codex"));
    }

    #[test]
    fn test_can_trigger_depth_exceeded() {
        let mut ctx = TriggerContext::new();
        ctx.depth = ctx.max_depth;
        assert!(!ctx.can_trigger("codex"));
    }

    #[test]
    fn test_child_for_increments_depth() {
        let ctx = TriggerContext::new();
        let child = ctx.child_for("codex");
        assert_eq!(child.depth, 1);
        assert!(child.triggered_agents.contains("codex"));
        // Parent should be unchanged
        assert_eq!(ctx.depth, 0);
        assert!(!ctx.triggered_agents.contains("codex"));
    }

    #[test]
    fn test_chain_depth_3_levels() {
        let ctx = TriggerContext::with_max_depth(3);

        // Level 1: A triggers B
        let ctx_b = ctx.child_for("agent_b");
        assert_eq!(ctx_b.depth, 1);
        assert!(ctx_b.can_trigger("agent_c"));

        // Level 2: B triggers C
        let ctx_c = ctx_b.child_for("agent_c");
        assert_eq!(ctx_c.depth, 2);
        assert!(ctx_c.can_trigger("agent_d"));

        // Level 3: C tries to trigger D -- but depth(3) >= max_depth(3)
        let ctx_d = ctx_c.child_for("agent_d");
        assert_eq!(ctx_d.depth, 3);
        // At depth 3, can_trigger should be false (depth >= max_depth)
        assert!(!ctx_d.can_trigger("agent_e"));
    }

    // ---- extract_valid_triggers tests ----

    #[test]
    fn test_extract_valid_triggers_basic() {
        let members = make_members(&["claude", "codex", "gemini"]);
        let ctx = TriggerContext::new();
        let triggers = extract_valid_triggers(
            "@Codex please review. @Gemini analyze.",
            &members,
            &ctx,
        );
        assert_eq!(triggers, vec!["codex", "gemini"]);
    }

    #[test]
    fn test_extract_valid_triggers_skips_already_triggered() {
        let members = make_members(&["claude", "codex"]);
        let mut ctx = TriggerContext::new();
        ctx.triggered_agents.insert("codex".to_string());
        let triggers = extract_valid_triggers(
            "@Codex please review.",
            &members,
            &ctx,
        );
        assert!(triggers.is_empty());
    }

    #[test]
    fn test_extract_valid_triggers_depth_limit() {
        let members = make_members(&["claude", "codex", "gemini"]);
        let mut ctx = TriggerContext::new();
        ctx.depth = ctx.max_depth; // Already at max
        let triggers = extract_valid_triggers(
            "@Codex please review.",
            &members,
            &ctx,
        );
        assert!(triggers.is_empty());
    }

    #[test]
    fn test_extract_valid_triggers_ignores_non_member() {
        let members = make_members(&["claude"]);
        let ctx = TriggerContext::new();
        let triggers = extract_valid_triggers(
            "@GPT-4 help please.",
            &members,
            &ctx,
        );
        assert!(triggers.is_empty());
    }

    // ---- classify_triggers tests ----

    #[test]
    fn test_classify_triggers_all_valid() {
        let members = make_members(&["claude", "codex", "gemini"]);
        let ctx = TriggerContext::new();
        let result = classify_triggers(
            "@Codex @Gemini review this.",
            &members,
            &ctx,
        );
        assert!(result.triggered.contains(&"codex".to_string()));
        assert!(result.triggered.contains(&"gemini".to_string()));
        assert!(result.depth_exceeded.is_empty());
        assert!(result.dedup_skipped.is_empty());
    }

    #[test]
    fn test_classify_triggers_depth_exceeded() {
        let members = make_members(&["claude", "codex"]);
        let mut ctx = TriggerContext::new();
        ctx.depth = ctx.max_depth;
        let result = classify_triggers(
            "@Codex help!",
            &members,
            &ctx,
        );
        assert!(result.triggered.is_empty());
        assert!(result.depth_exceeded.contains(&"codex".to_string()));
    }

    #[test]
    fn test_classify_triggers_dedup() {
        let members = make_members(&["claude", "codex"]);
        let mut ctx = TriggerContext::new();
        ctx.triggered_agents.insert("codex".to_string());
        let result = classify_triggers(
            "@Codex help!",
            &members,
            &ctx,
        );
        assert!(result.triggered.is_empty());
        assert!(result.dedup_skipped.contains(&"codex".to_string()));
    }

    #[test]
    fn test_classify_triggers_mixed() {
        let members = make_members(&["claude", "codex", "gemini"]);
        let mut ctx = TriggerContext::new();
        ctx.triggered_agents.insert("codex".to_string());
        ctx.depth = ctx.max_depth - 1; // Can still trigger one more level
        let result = classify_triggers(
            "@Codex @Gemini review this.",
            &members,
            &ctx,
        );
        // codex skipped (dedup), gemini is valid
        assert!(result.triggered.contains(&"gemini".to_string()));
        assert!(result.dedup_skipped.contains(&"codex".to_string()));
    }
}
