//! Zone Agent Protocol -- the L2 layer in the Prompt 7-layer architecture.
//!
//! When an Agent is triggered inside a Channel, this module renders a
//! structured context block that tells the Agent:
//! - who else is in the Channel (fellow Agent members),
//! - what each member's role and capabilities are,
//! - the collaboration rules for multi-Agent interactions,
//! - and how to use @agent mentions to trigger other Agents.
//!
//! This layer is **only** injected for Channel conversations, **not** for
//! Thread (1-on-1) conversations.

use crate::workspace::channel::Channel;
use crate::workspace::manager::Agent;

// ===========================================================================
// Data model
// // ===========================================================================

/// Information about a single Agent member, as rendered in the Zone Protocol.
#[derive(Debug, Clone)]
pub struct AgentMemberInfo {
    /// Agent identifier (directory name).
    pub agent_id: String,
    /// Display name (e.g. "Claude").
    pub display_name: String,
    /// Creature type (AI, robot, ghost, cat, etc.).
    pub creature: String,
    /// Personality vibe (sharp, warm, chaotic, calm).
    pub vibe: String,
    /// Human-readable role description derived from creature + vibe.
    pub role_description: String,
    /// Runtime type display name (e.g. "Claude Code", "OpenAI Codex").
    pub runtime_type: String,
}

/// The complete Zone Agent Protocol for a Channel.
///
/// Built from Channel + Agent identity data, and rendered as a prompt text
/// block that gets injected as L2 in the assembled system prompt.
#[derive(Debug, Clone)]
pub struct ChannelZoneProtocol {
    /// Channel display name.
    pub channel_name: String,
    /// Optional channel description.
    pub channel_description: Option<String>,
    /// The human user's display name in this channel.
    pub user_name: String,
    /// Agent members in this channel.
    pub members: Vec<AgentMemberInfo>,
}

// ===========================================================================
// Constructors
// // ===========================================================================

impl ChannelZoneProtocol {
    /// Build a Zone Protocol from a Channel, user name, and Agent records.
    ///
    /// The `agents` slice should contain **all** agents that are members of
    /// the channel.  Any channel member whose `agent_id` is not found in the
    /// agents list is silently skipped (defensive -- should not happen in
    /// normal operation).
    pub fn from_channel(channel: &Channel, user_name: &str, agents: &[Agent]) -> Self {
        let members: Vec<AgentMemberInfo> = channel
            .members
            .iter()
            .filter_map(|cm| {
                agents.iter().find(|a| a.agent_id == cm.agent_id).map(|a| {
                    let identity = &a.identity;
                    AgentMemberInfo {
                        agent_id: identity.agent_id.clone(),
                        display_name: identity.name.clone(),
                        creature: identity.creature.clone(),
                        vibe: identity.vibe.clone(),
                        role_description: derive_role_description(
                            &identity.creature,
                            &identity.vibe,
                        ),
                        runtime_type: identity.runtime_type.display_name().to_string(),
                    }
                })
            })
            .collect();

        Self {
            channel_name: channel.name.clone(),
            channel_description: None,
            user_name: user_name.to_string(),
            members,
        }
    }
}

// ===========================================================================
// Rendering
// // ===========================================================================

impl ChannelZoneProtocol {
    /// Render the Zone Agent Protocol as a Markdown prompt text block.
    ///
    /// The output is designed to be directly injected into the system prompt
    /// sent to the Agent runtime.
    pub fn render(&self) -> String {
        let mut out = String::with_capacity(1024);

        // Header
        out.push_str(&format!("## Channel: {}\n\n", self.channel_name));

        if let Some(ref desc) = self.channel_description {
            out.push_str(&format!("> {}\n\n", desc));
        }

        // Member table (includes the human user as first row)
        out.push_str("### Current Channel Members\n\n");
        out.push_str("| Member | Type | Role | Capabilities |\n");
        out.push_str("|--------|------|------|-------------|\n");

        // User row
        out.push_str(&format!(
            "| @{} | **User** | Human | The person you are helping |\n",
            self.user_name
        ));

        // Agent rows
        for m in &self.members {
            out.push_str(&format!(
                "| @{} | Agent | {} | {} |\n",
                m.display_name, m.role_description, m.vibe
            ));
        }
        out.push('\n');

        // Collaboration rules
        out.push_str("### Collaboration Rules\n\n");
        if self.members.len() <= 1 {
            out.push_str("You are currently the only Agent in this Channel. ");
            out.push_str("If other Agents are added later, the rules below will apply.\n\n");
        }
        out.push_str(&format!("1. The user's name is **@{}**. When you need to address or notify the user, ALWAYS use @{} format — this triggers a visual highlight and notification on their end.\n", self.user_name, self.user_name));
        out.push_str("2. You can mention other Agents with @AgentName in your reply -- the system will automatically trigger that Agent to respond.\n");
        out.push_str("3. If a task requires another Agent's expertise, proactively @mention them.\n");
        out.push_str("4. Be collaborative: if a question is better suited for another Agent, suggest that the user @mention that Agent.\n");
        out.push('\n');

        // @mention format
        out.push_str("### @Mention Format\n\n");
        out.push_str(&format!("- Address the user: @{}\n", self.user_name));
        out.push_str("- Mention an agent: @AgentName\n");
        out.push_str("- Mention with instruction: @Claude please review this code\n");
        out.push_str(&format!("- Notify the user of results: @{}, here is what I found...\n", self.user_name));

        // Task Suggestion Protocol
        out.push_str("\n### Task Suggestion Protocol\n\n");
        out.push_str("When you identify actionable tasks from the user's request, output them in the following format:\n\n");
        out.push_str("<task-suggestions>\n");
        out.push_str("[\n");
        out.push_str("  {\n");
        out.push_str("    \"title\": \"Task title\",\n");
        out.push_str("    \"description\": \"Brief description\",\n");
        out.push_str("    \"priority\": 3,\n");
        out.push_str("    \"assignee\": \"AgentName or null\",\n");
        out.push_str("    \"dependencies\": []\n");
        out.push_str("  }\n");
        out.push_str("]\n");
        out.push_str("</task-suggestions>\n\n");
        out.push_str("Rules:\n");
        out.push_str("- Only suggest tasks that are concrete and actionable\n");
        out.push_str("- Use priority 1 (critical) to 5 (trivial), default is 3\n");
        out.push_str("- Each task should be independently completable\n");
        out.push_str("- If no actionable tasks are identified, do NOT output this block\n");
        out.push_str("- You may output regular text before/after the block\n");

        out
    }
}

// ===========================================================================
// Helpers
// // ===========================================================================

/// Derive a short human-readable role description from creature + vibe.
fn derive_role_description(creature: &str, vibe: &str) -> String {
    // Simple heuristic: combine creature and vibe into a short role tag.
    // In the future this could read from IDENTITY.md or a dedicated field.
    match (creature.to_lowercase().as_str(), vibe.to_lowercase().as_str()) {
        ("ai", "sharp") => "Code Expert".to_string(),
        ("ai", "warm") => "Communication Specialist".to_string(),
        ("ai", "calm") => "Research Assistant".to_string(),
        ("ai", "chaotic") => "Creative Ideator".to_string(),
        ("ai", "helpful") => "General Assistant".to_string(),
        ("robot", _) => "Task Executor".to_string(),
        ("cat", _) => "Curious Observer".to_string(),
        ("ghost", _) => "Silent Analyst".to_string(),
        _ => format!("{} {}", capitalize(vibe), capitalize(creature)),
    }
}

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            upper + chars.as_str()
        }
    }
}

// ===========================================================================
// Tests
// // ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::channel::ChannelMember;
    use crate::workspace::identity::AgentIdentity;
    use crate::runtime::RuntimeType;

    /// Helper: create a test Channel with given agent IDs as members.
    fn make_channel(name: &str, agent_ids: &[&str]) -> Channel {
        let now = crate::workspace::channel::now_iso();
        Channel {
            id: crate::workspace::channel::generate_channel_id(),
            name: name.to_string(),
            members: agent_ids
                .iter()
                .map(|&aid| ChannelMember {
                    agent_id: aid.to_string(),
                    role: "member".to_string(),
                    joined_at: now.clone(),
                })
                .collect(),
            messages: Vec::new(),
            summary: None,
            summary_up_to: None,
            summary_updated_at: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Helper: create a test Agent with identity.
    fn make_agent(
        agent_id: &str,
        name: &str,
        creature: &str,
        vibe: &str,
        runtime_type: RuntimeType,
    ) -> Agent {
        Agent {
            agent_id: agent_id.to_string(),
            identity: AgentIdentity::with_runtime_type(
                agent_id,
                name,
                creature,
                vibe,
                "robot",
                runtime_type,
            ),
            enabled: true,
            session_count: 0,
        }
    }

    #[test]
    fn test_render_multi_agent_channel() {
        let channel = make_channel("Dev Team", &["claude", "codex"]);
        let agents = vec![
            make_agent("claude", "Claude", "AI", "sharp", RuntimeType::ClaudeCode),
            make_agent("codex", "Codex", "AI", "calm", RuntimeType::Codex),
        ];

        let zp = ChannelZoneProtocol::from_channel(&channel, "Ryan", &agents);
        let rendered = zp.render();

        assert!(rendered.contains("## Channel: Dev Team"));
        assert!(rendered.contains("@Ryan"));
        assert!(rendered.contains("@Claude"));
        assert!(rendered.contains("@Codex"));
        assert!(rendered.contains("Code Expert"));
        assert!(rendered.contains("Research Assistant"));
        assert!(rendered.contains("**User**"));
        assert!(rendered.contains("Collaboration Rules"));
        assert!(rendered.contains("@Mention Format"));
        // Multi-agent should NOT contain "only" text
        assert!(!rendered.contains("only Agent in this Channel"));
    }

    #[test]
    fn test_render_single_agent_channel() {
        let channel = make_channel("Solo", &["claude"]);
        let agents = vec![
            make_agent("claude", "Claude", "AI", "sharp", RuntimeType::ClaudeCode),
        ];

        let zp = ChannelZoneProtocol::from_channel(&channel, "Ryan", &agents);
        let rendered = zp.render();

        assert!(rendered.contains("## Channel: Solo"));
        assert!(rendered.contains("@Claude"));
        // Single-agent should contain "only" text
        assert!(rendered.contains("only Agent in this Channel"));
    }

    #[test]
    fn test_render_with_channel_description() {
        let mut channel = make_channel("Project X", &["claude"]);
        channel.summary = None; // not using summary as description
        let agents = vec![
            make_agent("claude", "Claude", "AI", "sharp", RuntimeType::ClaudeCode),
        ];

        let mut zp = ChannelZoneProtocol::from_channel(&channel, "Ryan", &agents);
        zp.channel_description = Some("A channel for Project X development".to_string());
        let rendered = zp.render();

        assert!(rendered.contains("A channel for Project X development"));
    }

    #[test]
    fn test_missing_agent_skipped() {
        let channel = make_channel("Test", &["claude", "unknown_agent"]);
        let agents = vec![
            make_agent("claude", "Claude", "AI", "sharp", RuntimeType::ClaudeCode),
        ];

        let zp = ChannelZoneProtocol::from_channel(&channel, "Ryan", &agents);
        // Only 1 member should be rendered (unknown_agent skipped)
        assert_eq!(zp.members.len(), 1);
        assert_eq!(zp.members[0].display_name, "Claude");
    }

    #[test]
    fn test_member_table_format() {
        let channel = make_channel("Test", &["claude"]);
        let agents = vec![
            make_agent("claude", "Claude", "AI", "sharp", RuntimeType::ClaudeCode),
        ];

        let zp = ChannelZoneProtocol::from_channel(&channel, "Ryan", &agents);
        let rendered = zp.render();

        // Should have table header and separator
        assert!(rendered.contains("| Member | Type | Role | Capabilities |"));
        assert!(rendered.contains("|--------|------|------|-------------|"));
        // Should have user row and Claude row
        assert!(rendered.contains("| @Ryan | **User** | Human | The person you are helping |"));
        assert!(rendered.contains("| @Claude | Agent | Code Expert | sharp |"));
    }

    #[test]
    fn test_derive_role_description() {
        assert_eq!(derive_role_description("AI", "sharp"), "Code Expert");
        assert_eq!(derive_role_description("AI", "calm"), "Research Assistant");
        assert_eq!(derive_role_description("robot", "anything"), "Task Executor");
        assert_eq!(derive_role_description("cat", "playful"), "Curious Observer");
        // Unknown combo: "Playful Unknown"
        assert_eq!(derive_role_description("unknown", "playful"), "Playful Unknown");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("a"), "A");
    }
}
