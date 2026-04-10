/**
 * Content block type definitions for structured message rendering.
 *
 * Extends the base message types to support rich content blocks
 * from Claude Code runtime responses including tool_use and tool_result.
 */

// ---------------------------------------------------------------------------
// Content Block Types
// ---------------------------------------------------------------------------

/** Base content block */
export interface BaseContentBlock {
  type: string;
}

/** Text content block */
export interface TextContentBlock extends BaseContentBlock {
  type: 'text';
  text: string;
}

/** Tool use content block */
export interface ToolUseContentBlock extends BaseContentBlock {
  type: 'tool_use';
  id: string;
  name: string;
  input: Record<string, unknown>;
}

/** Tool result content block */
export interface ToolResultContentBlock extends BaseContentBlock {
  type: 'tool_result';
  tool_use_id: string;
  content: string;
  is_error?: boolean;
}

/** Union of all content block types */
export type ContentBlock = TextContentBlock | ToolUseContentBlock | ToolResultContentBlock;

// ---------------------------------------------------------------------------
// Parsing utilities
// ---------------------------------------------------------------------------

/**
 * Parse raw content (string or ContentBlock[]) into structured blocks.
 * If the content is a plain string, returns a single text block.
 */
export function parseContent(content: string | ContentBlock[] | undefined): ContentBlock[] {
  if (!content) return [];
  if (Array.isArray(content)) return content;
  if (typeof content === 'string') {
    return [{ type: 'text', text: content }];
  }
  return [{ type: 'text', text: String(content) }];
}

/**
 * Extract all text from content blocks as a single string.
 */
export function extractTextFromBlocks(blocks: ContentBlock[]): string {
  return blocks
    .filter((b): b is TextContentBlock => b.type === 'text')
    .map((b) => b.text)
    .join('');
}

/**
 * Check if content blocks contain tool calls.
 */
export function hasToolCalls(blocks: ContentBlock[]): boolean {
  return blocks.some((b) => b.type === 'tool_use');
}

/**
 * Check if content is structured (ContentBlock[]) vs plain string.
 */
export function isStructuredContent(content: unknown): content is ContentBlock[] {
  return Array.isArray(content);
}
