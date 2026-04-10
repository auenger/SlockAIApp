//! @Mention parser for Channel messages.
//!
//! Parses `@AgentName` and `@{Agent Name}` patterns from messages
//! and resolves them to Agent IDs based on Channel membership.

use super::channel::ChannelMember;

// ===========================================================================
// Mention parsing
// ===========================================================================

/// A parsed mention extracted from a message.
#[derive(Debug, Clone, PartialEq)]
pub struct Mention {
    /// The raw text matched (e.g., "@Claude" or "@{Agent Name}").
    pub raw: String,
    /// The resolved agent_id (lowercase, underscored).
    pub agent_id: String,
}

/// Result of parsing mentions from a message.
#[derive(Debug, Clone)]
pub struct MentionResult {
    /// All mentions found in the message.
    pub mentions: Vec<Mention>,
    /// The message with mentions stripped of their @ prefix for processing.
    pub cleaned_message: String,
}

/// Parse @mentions from a message and resolve them against Channel members.
///
/// Supports two formats:
/// - `@AgentName` — word-boundary mention (alphanumeric + underscore + CJK)
/// - `@{Agent Name}` — braced mention (allows spaces)
///
/// Returns a `MentionResult` with resolved mentions. Mentions that don't
/// match any channel member are ignored.
pub fn parse_mentions(message: &str, members: &[ChannelMember]) -> MentionResult {
    let mut mentions = Vec::new();
    let mut processed = message.to_string();

    // Build a lookup: agent_id -> member, and also name-based lookups
    let member_lookup = build_member_lookup(members);

    // First pass: parse @{Name With Spaces} format (greedy, higher priority)
    processed = parse_braced_mentions(&processed, &member_lookup, &mut mentions);

    // Second pass: parse @WordFormat mentions
    processed = parse_word_mentions(&processed, &member_lookup, &mut mentions);

    // Deduplicate mentions by agent_id while preserving order
    let mut seen = std::collections::HashSet::new();
    mentions.retain(|m| seen.insert(m.agent_id.clone()));

    MentionResult {
        mentions,
        cleaned_message: processed.trim().to_string(),
    }
}

/// Extract agent IDs that are @mentioned in an Agent's response text.
///
/// This is used for Agent-to-Agent (A2A) triggering: after an Agent produces
/// a response, we parse its output for @{agent} mentions and return the
/// corresponding agent IDs (only those that are Channel members).
///
/// Returns an empty Vec if no valid member mentions are found.
pub fn extract_agent_triggers(response: &str, members: &[ChannelMember]) -> Vec<String> {
    let result = parse_mentions(response, members);
    result
        .mentions
        .into_iter()
        .map(|m| m.agent_id)
        .collect()
}

/// Resolve agent IDs from mentions.
///
/// Returns the list of agent IDs for matched mentions, in order.
/// If no mentions are found, returns the default agent ID (first member).
pub fn resolve_agents(mentions: &[Mention], members: &[ChannelMember]) -> Vec<String> {
    if mentions.is_empty() {
        // Default: return the first member
        members
            .first()
            .map(|m| m.agent_id.clone())
            .into_iter()
            .collect()
    } else {
        mentions.iter().map(|m| m.agent_id.clone()).collect()
    }
}

// ===========================================================================
// Internal helpers
// ===========================================================================

/// Member lookup structure for fast name-based resolution.
struct MemberLookup {
    /// agent_id -> display name mapping.
    by_id: std::collections::HashMap<String, String>,
    /// lowercase display name -> agent_id mapping.
    by_name: std::collections::HashMap<String, String>,
}

fn build_member_lookup(members: &[ChannelMember]) -> MemberLookup {
    let mut by_id = std::collections::HashMap::new();
    let mut by_name = std::collections::HashMap::new();

    for member in members {
        by_id.insert(member.agent_id.clone(), member.agent_id.clone());

        // Name-based lookup: agent_id parts (underscore-separated) form the name
        let display_name = id_to_display_name(&member.agent_id);
        by_name.insert(display_name.to_lowercase(), member.agent_id.clone());

        // Also allow direct agent_id lookup
        by_name.insert(member.agent_id.to_lowercase(), member.agent_id.clone());
    }

    MemberLookup { by_id, by_name }
}

/// Convert an agent_id like "claude" or "my_agent" to a display name like "Claude" or "My Agent".
fn id_to_display_name(agent_id: &str) -> String {
    agent_id
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    upper + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parse @{Name With Spaces} mentions from the message.
fn parse_braced_mentions(
    message: &str,
    lookup: &MemberLookup,
    mentions: &mut Vec<Mention>,
) -> String {
    let mut result = message.to_string();

    // Find all @{...} patterns
    let re = regex_lazy();
    let mut offset_adjustment = 0i64;

    for cap in re.find_iter(message) {
        let full_match = cap.as_str();
        // Extract the name inside braces
        if let Some(inner) = full_match.strip_prefix("@{").and_then(|s| s.strip_suffix('}')) {
            let name = inner.trim();
            if let Some(agent_id) = resolve_name(name, lookup) {
                let raw_start = cap.start() as i64 + offset_adjustment;
                let raw_end = cap.end() as i64 + offset_adjustment;

                mentions.push(Mention {
                    raw: full_match.to_string(),
                    agent_id: agent_id.clone(),
                });

                // Replace the @{Name} with just the name in the cleaned message
                if raw_start >= 0 && raw_end >= 0 {
                    let start = raw_start as usize;
                    let end = raw_end as usize;
                    if start <= result.len() && end <= result.len() {
                        let replacement = name.to_string();
                        offset_adjustment += replacement.len() as i64 - (end - start) as i64;
                        result.replace_range(start..end, &replacement);
                    }
                }
            }
        }
    }

    result
}

/// Parse @WordFormat mentions from the message.
fn parse_word_mentions(
    message: &str,
    lookup: &MemberLookup,
    mentions: &mut Vec<Mention>,
) -> String {
    let mut result = String::new();
    let mut remaining = message;
    let mut already_mentioned: std::collections::HashSet<String> = mentions
        .iter()
        .map(|m| m.agent_id.clone())
        .collect();

    while let Some(at_pos) = remaining.find('@') {
        // Push everything before the @
        result.push_str(&remaining[..at_pos]);
        let after_at = &remaining[at_pos + 1..];

        // Try to match a name starting after @
        if let Some((name, agent_id)) = try_match_name(after_at, lookup) {
            if !already_mentioned.contains(&agent_id) {
                mentions.push(Mention {
                    raw: format!("@{}", name),
                    agent_id: agent_id.clone(),
                });
                already_mentioned.insert(agent_id.clone());
            }
            result.push_str(&name);
            remaining = &after_at[name.len()..];
        } else {
            // Not a valid mention, keep the @
            result.push('@');
            remaining = after_at;
        }
    }

    result.push_str(remaining);
    result
}

/// Try to match a name at the start of the string against known members.
///
/// Greedily matches the longest possible name.
fn try_match_name(text: &str, lookup: &MemberLookup) -> Option<(String, String)> {
    let chars: Vec<char> = text.chars().collect();
    let max_char_len = chars.len().min(50); // Reasonable max name length

    for char_end in (1..=max_char_len).rev() {
        let candidate: String = chars[..char_end].iter().collect();

        // Check if this is a valid word boundary (next char is not alphanumeric/underscore/CJK)
        if char_end < chars.len() {
            let next_char = chars[char_end];
            if next_char.is_alphanumeric() || next_char == '_' || is_cjk(next_char) {
                continue;
            }
        }

        // Try to resolve
        if let Some(agent_id) = resolve_name(&candidate, lookup) {
            return Some((candidate, agent_id));
        }

        // Also try with trailing whitespace stripped
        let trimmed = candidate.trim_end();
        if trimmed.len() < candidate.len() {
            if let Some(agent_id) = resolve_name(trimmed, lookup) {
                return Some((trimmed.to_string(), agent_id));
            }
        }
    }

    None
}

/// Resolve a name string to an agent_id using the member lookup.
fn resolve_name(name: &str, lookup: &MemberLookup) -> Option<String> {
    let lower = name.to_lowercase().trim().to_string();
    if lower.is_empty() {
        return None;
    }

    // Direct name match
    if let Some(id) = lookup.by_name.get(&lower) {
        return Some(id.clone());
    }

    // Try replacing spaces with underscores
    let with_underscores = lower.replace(' ', "_");
    if let Some(id) = lookup.by_name.get(&with_underscores) {
        return Some(id.clone());
    }

    // Try as agent_id directly
    if let Some(id) = lookup.by_id.get(&lower) {
        return Some(id.clone());
    }

    None
}

/// Check if a character is a CJK character.
fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |   // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' |   // CJK Unified Ideographs Extension A
        '\u{2E80}'..='\u{2EFF}' |   // CJK Radicals Supplement
        '\u{3000}'..='\u{303F}' |   // CJK Symbols and Punctuation
        '\u{F900}'..='\u{FAFF}' |   // CJK Compatibility Ideographs
        '\u{AC00}'..='\u{D7AF}' |   // Hangul Syllables
        '\u{3040}'..='\u{309F}' |   // Hiragana
        '\u{30A0}'..='\u{30FF}'     // Katakana
    )
}

/// Lazy-initialized regex for @{...} pattern matching.
fn regex_lazy() -> &'static regex::Regex {
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"@\{[^}]+\}").unwrap())
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

    #[test]
    fn test_parse_simple_mention() {
        let members = make_members(&["claude", "alice"]);
        let result = parse_mentions("@claude please review this", &members);
        assert_eq!(result.mentions.len(), 1);
        assert_eq!(result.mentions[0].agent_id, "claude");
    }

    #[test]
    fn test_parse_multiple_mentions() {
        let members = make_members(&["claude", "alice"]);
        let result = parse_mentions("@claude @alice please review", &members);
        assert_eq!(result.mentions.len(), 2);
        assert_eq!(result.mentions[0].agent_id, "claude");
        assert_eq!(result.mentions[1].agent_id, "alice");
    }

    #[test]
    fn test_parse_braced_mention() {
        let members = make_members(&["claude"]);
        let result = parse_mentions("@{claude} please help", &members);
        assert_eq!(result.mentions.len(), 1);
        assert_eq!(result.mentions[0].agent_id, "claude");
    }

    #[test]
    fn test_no_mentions_returns_default() {
        let members = make_members(&["claude", "alice"]);
        let result = parse_mentions("hello everyone", &members);
        assert!(result.mentions.is_empty());

        let agents = resolve_agents(&result.mentions, &members);
        assert_eq!(agents, vec!["claude".to_string()]); // first member as default
    }

    #[test]
    fn test_unknown_mention_ignored() {
        let members = make_members(&["claude"]);
        let result = parse_mentions("@unknown_agent help", &members);
        assert!(result.mentions.is_empty());
    }

    #[test]
    fn test_case_insensitive() {
        let members = make_members(&["claude"]);
        let result = parse_mentions("@Claude help", &members);
        assert_eq!(result.mentions.len(), 1);
        assert_eq!(result.mentions[0].agent_id, "claude");
    }

    #[test]
    fn test_cjk_mention() {
        let members = make_members(&["claude"]);
        let result = parse_mentions("@claude 分析一下", &members);
        assert_eq!(result.mentions.len(), 1);
        assert_eq!(result.mentions[0].agent_id, "claude");
    }

    #[test]
    fn test_deduplication() {
        let members = make_members(&["claude"]);
        let result = parse_mentions("@claude @claude help", &members);
        assert_eq!(result.mentions.len(), 1);
    }

    #[test]
    fn test_resolve_agents_order() {
        let members = make_members(&["claude", "alice", "bob"]);
        let result = parse_mentions("@bob @alice review please", &members);
        let agents = resolve_agents(&result.mentions, &members);
        assert_eq!(agents, vec!["bob", "alice"]);
    }

    #[test]
    fn test_id_to_display_name() {
        assert_eq!(id_to_display_name("claude"), "Claude");
        assert_eq!(id_to_display_name("my_agent"), "My Agent");
        assert_eq!(id_to_display_name("default"), "Default");
    }

    // ---- A2A trigger extraction tests ----

    #[test]
    fn test_extract_agent_triggers_single() {
        let members = make_members(&["claude", "codex"]);
        let triggers = extract_agent_triggers(
            "I found a bug. @Codex please fix it.",
            &members,
        );
        assert_eq!(triggers, vec!["codex"]);
    }

    #[test]
    fn test_extract_agent_triggers_multiple() {
        let members = make_members(&["claude", "codex", "gemini"]);
        let triggers = extract_agent_triggers(
            "Let me ask @Codex and @Gemini about this.",
            &members,
        );
        assert_eq!(triggers, vec!["codex", "gemini"]);
    }

    #[test]
    fn test_extract_agent_triggers_ignores_non_member() {
        let members = make_members(&["claude", "codex"]);
        let triggers = extract_agent_triggers(
            "I'm not sure. @GPT-4 might know.",
            &members,
        );
        assert!(triggers.is_empty());
    }

    #[test]
    fn test_extract_agent_triggers_no_mentions() {
        let members = make_members(&["claude", "codex"]);
        let triggers = extract_agent_triggers(
            "This is a straightforward answer.",
            &members,
        );
        assert!(triggers.is_empty());
    }

    #[test]
    fn test_extract_agent_triggers_braced_format() {
        let members = make_members(&["claude", "codex"]);
        let triggers = extract_agent_triggers(
            "Let me delegate to @{Codex}.",
            &members,
        );
        assert_eq!(triggers, vec!["codex"]);
    }

    #[test]
    fn test_extract_agent_triggers_case_insensitive() {
        let members = make_members(&["claude", "codex"]);
        let triggers = extract_agent_triggers(
            "@CODEX should handle this.",
            &members,
        );
        assert_eq!(triggers, vec!["codex"]);
    }

    #[test]
    fn test_extract_agent_triggers_dedup() {
        let members = make_members(&["claude", "codex"]);
        let triggers = extract_agent_triggers(
            "@Codex please help. Also @Codex review this.",
            &members,
        );
        assert_eq!(triggers, vec!["codex"]);
    }
}
