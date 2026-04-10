/**
 * ToolCallBlock — Structured display for Claude Code tool calls.
 *
 * Renders tool invocations (Read, Edit, Bash, Write, Glob, Grep, etc.)
 * as collapsible cards with:
 * - Tool name with icon
 * - Parameter summary
 * - Execution status indicator
 * - Collapsible result area
 */

import React, { useState } from 'react';
import {
  FileText,
  Pencil,
  Terminal,
  FileOutput,
  Search,
  FolderSearch,
  ChevronDown,
  ChevronRight,
  Loader2,
  CheckCircle2,
  XCircle,
  Wrench,
} from 'lucide-react';
import { cn } from '../../lib/utils';

// ---------------------------------------------------------------------------
// Tool metadata helpers
// ---------------------------------------------------------------------------

interface ToolMeta {
  label: string;
  icon: React.ElementType;
  color: string;
}

const TOOL_META: Record<string, ToolMeta> = {
  Read: { label: 'Read file', icon: FileText, color: 'bg-blue-100 text-blue-700' },
  Edit: { label: 'Edit file', icon: Pencil, color: 'bg-orange-100 text-orange-700' },
  Write: { label: 'Write file', icon: FileOutput, color: 'bg-green-100 text-green-700' },
  Bash: { label: 'Run command', icon: Terminal, color: 'bg-purple-100 text-purple-700' },
  Glob: { label: 'Search files', icon: FolderSearch, color: 'bg-cyan-100 text-cyan-700' },
  Grep: { label: 'Search content', icon: Search, color: 'bg-pink-100 text-pink-700' },
};

function getToolMeta(toolName: string): ToolMeta {
  return TOOL_META[toolName] || { label: toolName, icon: Wrench, color: 'bg-gray-100 text-gray-700' };
}

/** Extract a brief parameter summary from tool input */
function getParamSummary(toolName: string, input: Record<string, unknown>): string {
  switch (toolName) {
    case 'Read':
      return String(input.file_path || input.path || '');
    case 'Edit': {
      const path = String(input.file_path || input.path || '');
      const oldText = input.old_string ? String(input.old_string) : '';
      const snippet = oldText.length > 40 ? oldText.substring(0, 40) + '...' : oldText;
      return `${path} — "${snippet}"`;
    }
    case 'Write':
      return String(input.file_path || input.path || '');
    case 'Bash':
      return String(input.command || '');
    case 'Glob':
      return String(input.pattern || '');
    case 'Grep': {
      const pattern = String(input.pattern || '');
      const path = input.path ? ` in ${input.path}` : '';
      return `"${pattern}"${path}`;
    }
    default:
      return '';
  }
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type ToolCallStatus = 'running' | 'completed' | 'error';

export interface ToolCallData {
  /** Unique identifier for this tool call */
  id: string;
  /** Tool name (e.g. "Read", "Edit", "Bash") */
  name: string;
  /** Tool input parameters */
  input: Record<string, unknown>;
  /** Current status */
  status: ToolCallStatus;
  /** Result content (when completed) */
  result?: string;
  /** Error message (when status is error) */
  error?: string;
}

// ---------------------------------------------------------------------------
// ToolCallBlock
// ---------------------------------------------------------------------------

interface ToolCallBlockProps {
  data: ToolCallData;
  className?: string;
}

export const ToolCallBlock: React.FC<ToolCallBlockProps> = ({ data, className }) => {
  const [expanded, setExpanded] = useState(false);
  const meta = getToolMeta(data.name);
  const Icon = meta.icon;
  const summary = getParamSummary(data.name, data.input);

  const statusIcon = data.status === 'running' ? (
    <Loader2 size={12} className="animate-spin text-brutal-cyan" />
  ) : data.status === 'completed' ? (
    <CheckCircle2 size={12} className="text-brutal-green" />
  ) : (
    <XCircle size={12} className="text-red-500" />
  );

  const statusLabel = data.status === 'running' ? 'Running'
    : data.status === 'completed' ? 'Done'
    : 'Error';

  return (
    <div className={cn("brutal-border bg-white my-2 overflow-hidden", className)}>
      {/* Header — always visible */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-2 px-3 py-2 text-left hover:bg-gray-50 transition-colors"
      >
        {expanded ? (
          <ChevronDown size={14} className="text-gray-400 shrink-0" />
        ) : (
          <ChevronRight size={14} className="text-gray-400 shrink-0" />
        )}
        <div className={cn("p-1 brutal-border", meta.color)}>
          <Icon size={12} />
        </div>
        <span className="font-black text-[10px] uppercase">{meta.label}</span>
        {summary && (
          <span className="text-[10px] text-gray-500 truncate flex-1 font-mono">
            {summary}
          </span>
        )}
        <div className="flex items-center gap-1 shrink-0 ml-2">
          {statusIcon}
          <span className={cn(
            "text-[8px] font-bold uppercase",
            data.status === 'running' ? "text-brutal-cyan" :
            data.status === 'completed' ? "text-brutal-green" : "text-red-500"
          )}>
            {statusLabel}
          </span>
        </div>
      </button>

      {/* Expandable content */}
      {expanded && (data.result || data.error) && (
        <div className="brutal-border-t">
          {data.error ? (
            <div className="px-3 py-2 bg-red-50 text-xs text-red-600 font-mono">
              {data.error}
            </div>
          ) : data.result ? (
            <div className="px-3 py-2 bg-gray-50 max-h-[300px] overflow-auto">
              <pre className="text-[10px] leading-relaxed font-mono whitespace-pre-wrap break-all">
                {data.result.length > 5000
                  ? data.result.substring(0, 5000) + '\n... (truncated)'
                  : data.result
                }
              </pre>
            </div>
          ) : null}
        </div>
      )}
    </div>
  );
};

// ---------------------------------------------------------------------------
// ToolResultBlock — for standalone tool results
// ---------------------------------------------------------------------------

interface ToolResultBlockProps {
  toolUseId: string;
  content: string;
  isError?: boolean;
  className?: string;
}

export const ToolResultBlock: React.FC<ToolResultBlockProps> = ({
  content,
  isError = false,
  className,
}) => {
  const [expanded, setExpanded] = useState(false);
  const lineCount = content.split('\n').length;

  return (
    <div className={cn("brutal-border bg-white my-2 overflow-hidden", className)}>
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-2 px-3 py-1.5 text-left hover:bg-gray-50 transition-colors"
      >
        {expanded ? (
          <ChevronDown size={12} className="text-gray-400" />
        ) : (
          <ChevronRight size={12} className="text-gray-400" />
        )}
        <span className={cn(
          "text-[9px] font-bold uppercase",
          isError ? "text-red-500" : "text-gray-500"
        )}>
          {isError ? 'Error' : 'Result'}
        </span>
        <span className="text-[9px] text-gray-400">
          {lineCount} line{lineCount !== 1 ? 's' : ''}
        </span>
      </button>
      {expanded && (
        <div className={cn(
          "brutal-border-t px-3 py-2 max-h-[300px] overflow-auto",
          isError ? "bg-red-50" : "bg-gray-50"
        )}>
          <pre className="text-[10px] leading-relaxed font-mono whitespace-pre-wrap break-all">
            {content.length > 5000
              ? content.substring(0, 5000) + '\n... (truncated)'
              : content
            }
          </pre>
        </div>
      )}
    </div>
  );
};
