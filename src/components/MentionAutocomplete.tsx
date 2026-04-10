/**
 * @Mention autocomplete component for Channel message input.
 *
 * When the user types '@' in the channel input, shows a dropdown of
 * available agent members. Filters by name as the user types more.
 */

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { cn } from '../lib/utils';
import type { AgentWithRuntime } from '../types';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface MentionAutocompleteProps {
  /** Current input value */
  value: string;
  /** Called when the value changes (after selecting a mention) */
  onChange: (value: string) => void;
  /** Agent members of the current channel */
  members: AgentWithRuntime[];
  /** Whether the input is disabled */
  disabled?: boolean;
  /** Placeholder text */
  placeholder?: string;
  /** Additional class names */
  className?: string;
  /** Callback on Enter key (for sending) */
  onSend?: () => void;
  /** Callback on Shift+Enter (for newline - default behavior) */
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const MentionAutocomplete: React.FC<MentionAutocompleteProps> = ({
  value,
  onChange,
  members,
  disabled = false,
  placeholder = '',
  className = '',
  onSend,
}) => {
  const [showDropdown, setShowDropdown] = useState(false);
  const [filter, setFilter] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [mentionStart, setMentionStart] = useState(-1);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Filter members by the current @mention query
  const filteredMembers = members.filter((awr) => {
    const name = awr.agent.name.toLowerCase();
    const id = awr.agent.agent_id.toLowerCase();
    const q = filter.toLowerCase();
    return name.includes(q) || id.includes(q);
  });

  // Detect @mention being typed
  const handleInput = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newValue = e.target.value;
    const cursorPos = e.target.selectionStart;

    onChange(newValue);

    // Check if cursor is inside a @mention
    const textBeforeCursor = newValue.substring(0, cursorPos);
    const lastAtIndex = textBeforeCursor.lastIndexOf('@');

    if (lastAtIndex !== -1) {
      // Check if the @ is at the start or preceded by whitespace
      if (lastAtIndex === 0 || /\s/.test(textBeforeCursor[lastAtIndex - 1])) {
        const mentionText = textBeforeCursor.substring(lastAtIndex + 1);
        // Only show dropdown if mention text doesn't contain spaces (still typing)
        if (!mentionText.includes(' ') && mentionText.length < 30) {
          setFilter(mentionText);
          setMentionStart(lastAtIndex);
          setShowDropdown(true);
          setSelectedIndex(0);
          return;
        }
      }
    }

    setShowDropdown(false);
    setMentionStart(-1);
  }, [onChange]);

  // Insert a mention
  const insertMention = useCallback((awr: AgentWithRuntime) => {
    const cursorPos = textareaRef.current?.selectionStart ?? value.length;
    if (mentionStart === -1) return;

    const before = value.substring(0, mentionStart);
    const after = value.substring(cursorPos);
    const mention = `@${awr.agent.name} `;
    const newValue = before + mention + after;

    onChange(newValue);
    setShowDropdown(false);
    setMentionStart(-1);
    setFilter('');

    // Set cursor position after the mention
    requestAnimationFrame(() => {
      if (textareaRef.current) {
        const newPos = before.length + mention.length;
        textareaRef.current.setSelectionRange(newPos, newPos);
        textareaRef.current.focus();
      }
    });
  }, [value, mentionStart, onChange]);

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (showDropdown && filteredMembers.length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedIndex((prev) => (prev + 1) % filteredMembers.length);
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedIndex((prev) => (prev - 1 + filteredMembers.length) % filteredMembers.length);
        return;
      }
      if (e.key === 'Tab' || e.key === 'Enter') {
        e.preventDefault();
        insertMention(filteredMembers[selectedIndex]);
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        setShowDropdown(false);
        return;
      }
    }

    // Normal key handling
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      onSend?.();
    }
  }, [showDropdown, filteredMembers, selectedIndex, insertMention, onSend]);

  // Close dropdown on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node) &&
          textareaRef.current && !textareaRef.current.contains(e.target as Node)) {
        setShowDropdown(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  return (
    <div className="relative">
      <textarea
        ref={textareaRef}
        value={value}
        onChange={handleInput}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={disabled}
        className={cn(
          "w-full brutal-border bg-white p-3 min-h-[100px] text-sm focus:outline-none focus:bg-brutal-bg resize-none",
          disabled && "opacity-50 cursor-not-allowed",
          className
        )}
      />

      {/* @Mention dropdown */}
      {showDropdown && filteredMembers.length > 0 && (
        <div
          ref={dropdownRef}
          className="absolute bottom-full left-0 mb-1 w-64 max-h-48 overflow-y-auto bg-white brutal-border brutal-shadow-sm z-50"
        >
          <div className="p-1.5 brutal-border-b bg-gray-50">
            <span className="text-[9px] font-black uppercase text-gray-500">
              @ Mention an Agent
            </span>
          </div>
          {filteredMembers.map((awr, idx) => (
            <button
              key={awr.agent.agent_id}
              onClick={() => insertMention(awr)}
              className={cn(
                "w-full text-left px-3 py-2 flex items-center gap-2 text-xs transition-colors",
                idx === selectedIndex
                  ? "bg-brutal-pink text-white"
                  : "hover:bg-gray-100"
              )}
              onMouseEnter={() => setSelectedIndex(idx)}
            >
              <div className={cn(
                "w-6 h-6 brutal-border flex items-center justify-center shrink-0 text-sm",
                idx === selectedIndex ? "bg-white/20" : "bg-brutal-cyan"
              )}>
                {awr.agent.emoji}
              </div>
              <div className="flex-1 min-w-0">
                <div className={cn(
                  "font-black truncate flex items-center gap-1.5",
                  idx === selectedIndex ? "text-white" : ""
                )}>
                  {awr.agent.name}
                  <span className={cn(
                    "text-[8px] font-mono px-1 py-0 rounded-sm",
                    idx === selectedIndex
                      ? "bg-white/20 text-white/80"
                      : "bg-gray-100 text-gray-400"
                  )}>
                    {awr.runtime_type === 'claude_code' ? 'Claude Code'
                      : awr.runtime_type === 'codex' ? 'Codex'
                      : awr.runtime_type === 'gemini' ? 'Gemini'
                      : String(awr.runtime_type)}
                  </span>
                </div>
                <div className={cn(
                  "text-[9px] truncate",
                  idx === selectedIndex ? "text-white/70" : "text-gray-500"
                )}>
                  @{awr.agent.agent_id}
                </div>
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
};

// ---------------------------------------------------------------------------
// @Mention text highlighting utility
// ---------------------------------------------------------------------------

/**
 * Renders message content with @mentions highlighted as colored agent pills.
 * Returns an array of React nodes.
 */
export function renderMentionText(
  content: string,
  agents: AgentWithRuntime[],
  mentionClassName?: string,
  agentColorMap?: Map<string, number>
): React.ReactNode[] {
  const parts: React.ReactNode[] = [];

  // Build lookup from agent names/ids
  const agentLookup = agents.map((awr) => ({
    id: awr.agent.agent_id,
    name: awr.agent.name,
    emoji: awr.agent.emoji || '',
    awr,
  }));

  // Find all @Word patterns
  const mentionRegex = /@(\S+)/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;
  let keyIdx = 0;

  const colorList = [
    'bg-brutal-cyan', 'bg-brutal-pink', 'bg-brutal-yellow',
    'bg-purple-400', 'bg-brutal-green', 'bg-orange-400',
    'bg-teal-400', 'bg-red-400',
  ];

  while ((match = mentionRegex.exec(content)) !== null) {
    // Push text before the mention
    if (match.index > lastIndex) {
      parts.push(
        <span key={keyIdx++}>{content.substring(lastIndex, match.index)}</span>
      );
    }

    const fullMatch = match[0]; // e.g., "@Claude"
    const mentionName = match[1]; // e.g., "Claude"

    // Check if this matches an agent
    const matched = agentLookup.find(
      (a) => a.name.toLowerCase() === mentionName.toLowerCase() ||
              a.id.toLowerCase() === mentionName.toLowerCase()
    );

    if (matched) {
      const colorIdx = agentColorMap?.get(matched.id) ?? 0;
      const bgColor = colorList[colorIdx % colorList.length];
      const emoji = matched.emoji ? `${matched.emoji} ` : '';
      parts.push(
        <span
          key={keyIdx++}
          className={`inline-flex items-center gap-0.5 ${bgColor} text-black font-bold px-1.5 py-0 rounded-sm text-xs border border-black`}
        >
          {emoji}{matched.name}
        </span>
      );
    } else if (mentionClassName) {
      // Fallback to legacy style if className provided
      parts.push(
        <span key={keyIdx++} className={mentionClassName}>
          {fullMatch}
        </span>
      );
    } else {
      parts.push(<span key={keyIdx++}>{fullMatch}</span>);
    }

    lastIndex = match.index + match[0].length;
  }

  // Push remaining text
  if (lastIndex < content.length) {
    parts.push(<span key={keyIdx++}>{content.substring(lastIndex)}</span>);
  }

  return parts.length > 0 ? parts : [content];
}
