/**
 * @Mention autocomplete component for Channel message input.
 *
 * When the user types '@' in the channel input, shows a dropdown of
 * available agent members. Filters by name as the user types more.
 */

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { cn } from '../lib/utils';
import type { AgentWithRuntime } from '../types';
import { AgentIcon } from './AgentIcon';

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
  /** Current user's display name (shown as "You" in dropdown) */
  userName?: string;
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
  userName,
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

  // Insert a user self-mention (@UserName)
  const insertUserMention = useCallback(() => {
    const cursorPos = textareaRef.current?.selectionStart ?? value.length;
    if (mentionStart === -1 || !userName) return;

    const before = value.substring(0, mentionStart);
    const after = value.substring(cursorPos);
    const mention = `@${userName} `;
    const newValue = before + mention + after;

    onChange(newValue);
    setShowDropdown(false);
    setMentionStart(-1);
    setFilter('');

    requestAnimationFrame(() => {
      if (textareaRef.current) {
        const newPos = before.length + mention.length;
        textareaRef.current.setSelectionRange(newPos, newPos);
        textareaRef.current.focus();
      }
    });
  }, [value, mentionStart, onChange, userName]);

  // Whether the User option should be shown (when userName is set)
  const showUserOption = showDropdown && !!userName;

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    const totalOptions = (showUserOption ? 1 : 0) + filteredMembers.length;
    if (showDropdown && totalOptions > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedIndex((prev) => (prev + 1) % totalOptions);
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedIndex((prev) => (prev - 1 + totalOptions) % totalOptions);
        return;
      }
      if (e.key === 'Tab' || e.key === 'Enter') {
        e.preventDefault();
        // Index 0 = User option (if shown), otherwise agents start at 0
        if (showUserOption && selectedIndex === 0) {
          insertUserMention();
        } else {
          const agentIdx = showUserOption ? selectedIndex - 1 : selectedIndex;
          insertMention(filteredMembers[agentIdx]);
        }
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
  }, [showDropdown, filteredMembers, selectedIndex, insertMention, insertUserMention, onSend, showUserOption]);

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
      {showDropdown && (filteredMembers.length > 0 || showUserOption) && (
        <div
          ref={dropdownRef}
          className="absolute bottom-full left-0 mb-1 w-72 max-h-56 overflow-y-auto bg-white border-2 border-black shadow-[4px_4px_0_0_rgba(0,0,0,1)] z-50"
        >
          <div className="px-3 py-2 border-b-2 border-black bg-brutal-yellow">
            <span className="text-[9px] font-black uppercase text-black">
              @ Mention
            </span>
          </div>

          {/* User self-mention option (index 0) */}
          {showUserOption && (
            <button
              onClick={insertUserMention}
              className={cn(
                "w-full text-left px-3 py-2.5 flex items-center gap-2 text-xs transition-all border-b-2 border-black",
                selectedIndex === 0
                  ? "bg-purple-500 text-white"
                  : "bg-purple-50 hover:bg-purple-100"
              )}
              onMouseEnter={() => setSelectedIndex(0)}
            >
              <div className={cn(
                "w-7 h-7 flex items-center justify-center shrink-0 text-[10px] font-black border-2 border-black",
                selectedIndex === 0
                  ? "bg-white text-purple-600"
                  : "bg-purple-400 text-white"
              )}>
                U
              </div>
              <div className="flex-1 min-w-0">
                <div className={cn(
                  "font-black truncate flex items-center gap-1.5",
                  selectedIndex === 0 ? "text-white" : "text-black"
                )}>
                  {userName}
                  <span className={cn(
                    "text-[8px] font-black px-1 py-0 border",
                    selectedIndex === 0
                      ? "bg-white/30 border-white/50 text-white"
                      : "bg-purple-200 border-purple-400 text-purple-700"
                  )}>
                    YOU
                  </span>
                </div>
                <div className={cn(
                  "text-[9px] font-mono truncate",
                  selectedIndex === 0 ? "text-white/70" : "text-gray-500"
                )}>
                  @{userName} · self-mention
                </div>
              </div>
            </button>
          )}

          {filteredMembers.map((awr, idx) => {
            const displayIdx = showUserOption ? idx + 1 : idx;
            const isSelected = displayIdx === selectedIndex;
            return (
              <button
                key={awr.agent.agent_id}
                onClick={() => insertMention(awr)}
                className={cn(
                  "w-full text-left px-3 py-2.5 flex items-center gap-2 text-xs transition-all border-b border-gray-200",
                  isSelected
                    ? "bg-brutal-pink text-white border-b-black"
                    : "hover:bg-brutal-bg"
                )}
                onMouseEnter={() => setSelectedIndex(displayIdx)}
              >
                <AgentIcon
                  icon={awr.agent.icon}
                  emoji={awr.agent.emoji}
                  size="sm"
                  bgColor={isSelected ? "bg-white/20" : "bg-brutal-cyan"}
                />
                <div className="flex-1 min-w-0">
                  <div className={cn(
                    "font-black truncate flex items-center gap-1.5",
                    isSelected ? "text-white" : "text-black"
                  )}>
                    {awr.agent.emoji && <span>{awr.agent.emoji}</span>}
                    {awr.agent.name}
                    <span className={cn(
                      "text-[8px] font-mono font-black px-1 py-0 border",
                      isSelected
                        ? "bg-white/20 border-white/50 text-white/90"
                        : "bg-gray-100 border-gray-300 text-gray-500"
                    )}>
                      {awr.runtime_type === 'claude_code' ? 'Claude Code'
                        : awr.runtime_type === 'codex' ? 'Codex'
                        : awr.runtime_type === 'gemini' ? 'Gemini'
                        : String(awr.runtime_type)}
                    </span>
                  </div>
                  <div className={cn(
                    "text-[9px] truncate",
                    displayIdx === selectedIndex ? "text-white/70" : "text-gray-500"
                  )}>
                    @{awr.agent.agent_id}
                  </div>
                </div>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
};

// ---------------------------------------------------------------------------
// @Mention text highlighting utility
// ---------------------------------------------------------------------------

/**
 * Renders message content with @mentions highlighted as styled badges.
 * Supports `@AgentName` format only.
 * Returns an array of React nodes.
 */
export function renderMentionText(
  content: string,
  agents: AgentWithRuntime[],
  mentionClassName?: string,
  agentColorMap?: Map<string, number>,
  userName?: string,
): React.ReactNode[] {
  const parts: React.ReactNode[] = [];

  // Build lookup from agent names/ids + user
  const agentLookup = [
    ...agents.map((awr) => ({
      id: awr.agent.agent_id,
      name: awr.agent.name,
      emoji: awr.agent.emoji || '',
      awr,
      isUser: false as const,
    })),
    // Include user as a mentionable entity (purple badge)
    ...(userName ? [{
      id: '__user__',
      name: userName,
      emoji: '',
      isUser: true as const,
    }] : []),
  ];

  const colorList = [
    'bg-brutal-cyan', 'bg-brutal-pink', 'bg-brutal-yellow',
    'bg-purple-400', 'bg-brutal-green', 'bg-orange-400',
    'bg-teal-400', 'bg-red-400',
  ];

  let keyIdx = 0;
  let lastIndex = 0;

  // Match @Word (letters, digits, dots, dashes, underscores)
  const mentionRegex = /@([\w.-]+)/g;
  let match: RegExpExecArray | null;

  while ((match = mentionRegex.exec(content)) !== null) {
    // Push text before the mention
    if (match.index > lastIndex) {
      parts.push(
        <span key={keyIdx++}>{content.substring(lastIndex, match.index)}</span>
      );
    }

    const fullMatch = match[0];
    const mentionName = match[1];

    // Check if this matches an agent
    const matched = agentLookup.find(
      (a) => a.name.toLowerCase() === mentionName.toLowerCase() ||
              a.id.toLowerCase() === mentionName.toLowerCase()
    );

    if (matched) {
      if (matched.isUser) {
        // User mention — purple badge
        parts.push(
          <span
            key={keyIdx++}
            className="inline-flex items-center gap-0.5 bg-purple-400 text-white font-bold px-1.5 py-0.5 text-xs border-2 border-black"
          >
            {matched.name}
          </span>
        );
      } else {
        const colorIdx = agentColorMap?.get(matched.id) ?? 0;
        const bgColor = colorList[colorIdx % colorList.length];
        const emoji = matched.emoji ? `${matched.emoji} ` : '';

        parts.push(
          <span
            key={keyIdx++}
            className={`inline-flex items-center gap-0.5 ${bgColor} text-black font-bold px-1.5 py-0.5 text-xs border-2 border-black`}
          >
            {emoji}{matched.name}
          </span>
        );
      }
    } else if (mentionClassName) {
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
