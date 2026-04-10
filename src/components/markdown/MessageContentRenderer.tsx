/**
 * MessageContentRenderer — Renders message content that may contain
 * both markdown text and tool call blocks.
 *
 * Handles:
 * - Plain text (string) content
 * - Markdown text
 * - Tool use blocks (collapsible cards)
 * - Tool result blocks (collapsible results)
 */

import React from 'react';
import { MarkdownRenderer } from './MarkdownRenderer';
import { ToolCallBlock, ToolResultBlock } from './ToolCallBlock';
import type {
  ContentBlock,
  TextContentBlock,
  ToolUseContentBlock,
  ToolResultContentBlock,
} from './types';
import { parseContent } from './types';
// Re-export for convenience
export type { ContentBlock, TextContentBlock, ToolUseContentBlock, ToolResultContentBlock } from './types';
export { parseContent, extractTextFromBlocks, hasToolCalls, isStructuredContent } from './types';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface MessageContentRendererProps {
  /** Message content — can be a plain string or structured content blocks */
  content: string | ContentBlock[];
  /** Whether we're in channel mode (affects mention rendering) */
  isChannelMode?: boolean;
  /** All agents (for resolving mentions) */
  allAgents?: Array<{ agent: { agent_id: string; name: string; emoji?: string; icon?: string | null } }>;
  /** Color map for agents */
  agentColorMap?: Map<string, number>;
  /** Optional mention renderer */
  mentionRenderer?: (text: string) => React.ReactNode;
  /** Compact mode */
  compact?: boolean;
  /** Additional className */
  className?: string;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const MessageContentRenderer: React.FC<MessageContentRendererProps> = ({
  content,
  compact = false,
  className,
}) => {
  const blocks = parseContent(content);

  if (blocks.length === 0) {
    return null;
  }

  // Simple text-only case — just render markdown
  const hasOnlyText = blocks.every((b) => b.type === 'text');
  if (hasOnlyText) {
    const text = blocks.map((b) => (b as TextContentBlock).text).join('');
    return (
      <MarkdownRenderer content={text} compact={compact} className={className} />
    );
  }

  // Mixed content — render each block
  return (
    <div className={className}>
      {blocks.map((block, idx) => {
        switch (block.type) {
          case 'text':
            return (
              <MarkdownRenderer
                key={idx}
                content={(block as TextContentBlock).text}
                compact={compact}
              />
            );
          case 'tool_use': {
            const toolBlock = block as ToolUseContentBlock;
            // Find the corresponding tool_result block
            const resultBlock = blocks.find(
              (b): b is ToolResultContentBlock =>
                b.type === 'tool_result' &&
                b.tool_use_id === toolBlock.id
            );

            return (
              <ToolCallBlock
                key={toolBlock.id || idx}
                data={{
                  id: toolBlock.id || `tool-${idx}`,
                  name: toolBlock.name,
                  input: toolBlock.input,
                  status: resultBlock ? 'completed' : 'running',
                  result: resultBlock?.content,
                  error: resultBlock?.is_error ? resultBlock.content : undefined,
                }}
              />
            );
          }
          case 'tool_result': {
            const resultBlock = block as ToolResultContentBlock;
            // Only render standalone if the corresponding tool_use wasn't in the same blocks
            const hasMatchingToolUse = blocks.some(
              (b): b is ToolUseContentBlock =>
                b.type === 'tool_use' && b.id === resultBlock.tool_use_id
            );
            if (hasMatchingToolUse) return null; // Already rendered inside ToolCallBlock
            return (
              <ToolResultBlock
                key={idx}
                toolUseId={resultBlock.tool_use_id}
                content={resultBlock.content}
                isError={resultBlock.is_error}
              />
            );
          }
          default:
            return null;
        }
      })}
    </div>
  );
};
