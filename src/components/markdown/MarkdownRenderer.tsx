/**
 * MarkdownRenderer — Unified Markdown rendering component.
 *
 * Uses react-markdown with remark-gfm for GitHub Flavored Markdown support.
 * Integrates CodeBlock for syntax highlighting and preserves @mention rendering.
 * Designed for Neo-Brutalism styling.
 */

import React, { useMemo } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw';
import { cn } from '../../lib/utils';
import { CodeBlock, InlineCode } from './CodeBlock';
import type { AgentWithRuntime } from '../../types';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface MarkdownRendererProps {
  /** The markdown content to render */
  content: string;
  /** Optional className for the wrapper */
  className?: string;
  /** Whether to render in compact mode (smaller text) */
  compact?: boolean;
  /** Callback for rendering @mentions (passed from parent) */
  mentionRenderer?: (text: string) => React.ReactNode;
  /** All agents (for resolving mentions) */
  allAgents?: AgentWithRuntime[];
  /** Color index map per agent_id for consistent mention coloring */
  agentColorMap?: Map<string, number>;
  /** Current user's display name (for rendering @UserName mentions from agents) */
  userName?: string;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const MarkdownRenderer: React.FC<MarkdownRendererProps> = ({
  content,
  className,
  compact = false,
  mentionRenderer,
  allAgents,
  agentColorMap,
  userName,
}) => {
  // Pre-process content: ensure it's a string
  const rawContent = typeof content === 'string' ? content : String(content ?? '');

  // Pre-process mentions into HTML spans before markdown parsing.
  // This allows @mentions inside agent messages to render with special styling.
  const safeContent = useMemo(() => {
    if ((!allAgents || allAgents.length === 0) && !userName) return rawContent;

    let processed = rawContent;
    const colorList = [
      'bg-brutal-cyan', 'bg-brutal-pink', 'bg-brutal-yellow',
      'bg-purple-400', 'bg-brutal-green', 'bg-orange-400',
      'bg-teal-400', 'bg-red-400',
    ];

    // Build lookup: agents + user
    const lookup = [
      ...(allAgents || []).map((awr) => ({
        id: awr.agent.agent_id,
        name: awr.agent.name,
        emoji: awr.agent.emoji || '',
      })),
      // Include user as a mentionable entity (purple badge)
      ...(userName ? [{
        id: '__user__',
        name: userName,
        emoji: '',
        isUser: true,
      }] : []),
    ];

    // Replace @Word mentions (known agents + user)
    processed = processed.replace(
      /@([\w.-]+)/g,
      (full, name) => {
        const matched = lookup.find(
          (a) => a.name.toLowerCase() === name.toLowerCase() ||
                  a.id.toLowerCase() === name.toLowerCase()
        );
        if (matched) {
          if ((matched as { isUser?: boolean }).isUser) {
            // User mention — purple badge (matches "YOU" styling)
            return `<span class="inline-flex items-center gap-0.5 bg-purple-400 text-white font-bold px-1.5 py-0.5 text-xs border-2 border-black">${matched.name}</span>`;
          }
          const colorIdx = agentColorMap?.get(matched.id) ?? 0;
          const bgColor = colorList[colorIdx % colorList.length];
          const emoji = matched.emoji ? `${matched.emoji} ` : '';
          return `<span class="inline-flex items-center gap-0.5 ${bgColor} text-black font-bold px-1.5 py-0.5 text-xs border-2 border-black">${emoji}${matched.name}</span>`;
        }
        return full; // Not an agent/user mention, leave as-is
      }
    );

    return processed;
  }, [rawContent, allAgents, agentColorMap, userName]);

  // Whether mentions were injected (always needs ReactMarkdown to render HTML spans)
  const hasMentionHtml = useMemo(() => {
    return safeContent !== rawContent;
  }, [safeContent, rawContent]);

  // Check if the content looks like it has any markdown at all
  const hasMarkdown = useMemo(() => {
    if (!safeContent) return false;
    // Check for common markdown patterns
    return (
      /[*_`~]/g.test(safeContent) || // bold, italic, code, strikethrough
      /^#{1,6}\s/gm.test(safeContent) || // headings
      /^\s*[-*+]\s/gm.test(safeContent) || // unordered lists
      /^\s*\d+\.\s/gm.test(safeContent) || // ordered lists
      /^\s*>/gm.test(safeContent) || // blockquotes
      /\[.*\]\(.*\)/g.test(safeContent) || // links
      /```[\s\S]*?```/g.test(safeContent) || // code blocks
      /^\|.+\|$/gm.test(safeContent) || // tables
      /^---$/gm.test(safeContent) // horizontal rules
    );
  }, [safeContent]);

  // If no markdown detected and no mention HTML injected, render as plain text
  if (!hasMarkdown && !mentionRenderer && !hasMentionHtml) {
    return (
      <div className={cn(
        compact ? "text-xs" : "text-sm",
        "leading-relaxed whitespace-pre-wrap",
        className
      )}>
        {safeContent}
      </div>
    );
  }

  return (
    <div className={cn(
      "markdown-body",
      compact ? "text-xs" : "text-sm",
      "leading-relaxed",
      className
    )}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeRaw]}
        components={{
          // Code blocks and inline code
          code({ className: codeClassName, children }) {
            const match = /language-(\w+)/.exec(codeClassName || '');
            const isInline = !codeClassName && !String(children).includes('\n');

            if (isInline) {
              return <InlineCode>{children}</InlineCode>;
            }

            return (
              <CodeBlock language={match?.[1] || ''}>
                {String(children).replace(/\n$/, '')}
              </CodeBlock>
            );
          },

          // Pre tag: pass through (code handles it)
          pre({ children }) {
            return <>{children}</>;
          },

          // Headings
          h1: ({ children }) => (
            <h1 className={cn(
              "font-black mt-4 mb-2 brutal-border-b pb-1",
              compact ? "text-lg" : "text-xl"
            )}>
              {children}
            </h1>
          ),
          h2: ({ children }) => (
            <h2 className={cn(
              "font-black mt-3 mb-1.5",
              compact ? "text-base" : "text-lg"
            )}>
              {children}
            </h2>
          ),
          h3: ({ children }) => (
            <h3 className={cn(
              "font-bold mt-2 mb-1",
              compact ? "text-sm" : "text-base"
            )}>
              {children}
            </h3>
          ),
          h4: ({ children }) => (
            <h4 className={cn("font-bold mt-2 mb-1", compact ? "text-xs" : "text-sm")}>
              {children}
            </h4>
          ),
          h5: ({ children }) => (
            <h5 className={cn("font-bold mt-1 mb-0.5 text-xs uppercase")}>
              {children}
            </h5>
          ),
          h6: ({ children }) => (
            <h6 className={cn("font-bold mt-1 mb-0.5 text-[10px] uppercase text-gray-600")}>
              {children}
            </h6>
          ),

          // Paragraphs
          p: ({ children }) => (
            <p className={cn("mb-2 last:mb-0", compact ? "text-xs" : "text-sm", "leading-relaxed")}>
              {children}
            </p>
          ),

          // Bold
          strong: ({ children }) => (
            <strong className="font-black">{children}</strong>
          ),

          // Italic
          em: ({ children }) => (
            <em className="italic">{children}</em>
          ),

          // Links
          a: ({ href, children }) => (
            <a
              href={href}
              target="_blank"
              rel="noopener noreferrer"
              className="text-brutal-cyan underline font-bold hover:bg-brutal-cyan/10"
            >
              {children}
            </a>
          ),

          // Blockquote
          blockquote: ({ children }) => (
            <blockquote className="pl-3 brutal-border-l-4 border-l-black bg-gray-50 py-2 pr-3 my-2">
              {children}
            </blockquote>
          ),

          // Unordered list
          ul: ({ children }) => (
            <ul className={cn("ml-4 space-y-1 my-2", compact ? "text-xs" : "text-sm")}>
              {children}
            </ul>
          ),

          // Ordered list
          ol: ({ children }) => (
            <ol className={cn("ml-4 space-y-1 my-2 list-decimal", compact ? "text-xs" : "text-sm")}>
              {children}
            </ol>
          ),

          // List item
          li: ({ children, ...rest }) => {
            const checked = (rest as Record<string, unknown>).checked as boolean | undefined;
            return (
            <li className="flex items-start gap-1.5">
              {checked !== undefined && checked !== null ? (
                <span className={cn(
                  "mt-0.5 w-3 h-3 brutal-border flex items-center justify-center shrink-0",
                  checked ? "bg-brutal-green" : "bg-white"
                )}>
                  {checked && <span className="text-[8px] font-black">X</span>}
                </span>
              ) : (
                <span className="mt-1 w-1.5 h-1.5 bg-black shrink-0" />
              )}
              <span className="flex-1">{children}</span>
            </li>
            );
          },

          // Table
          table: ({ children }) => (
            <div className="overflow-x-auto my-2 brutal-border">
              <table className="w-full text-xs">
                {children}
              </table>
            </div>
          ),

          // Table head
          thead: ({ children }) => (
            <thead className="bg-gray-100">{children}</thead>
          ),

          // Table body
          tbody: ({ children }) => (
            <tbody>{children}</tbody>
          ),

          // Table row
          tr: ({ children }) => (
            <tr className="brutal-border-b">{children}</tr>
          ),

          // Table header cell
          th: ({ children }) => (
            <th className="px-3 py-2 text-left font-black text-[10px] uppercase brutal-border-r">
              {children}
            </th>
          ),

          // Table data cell
          td: ({ children }) => (
            <td className="px-3 py-2 brutal-border-r text-xs">
              {children}
            </td>
          ),

          // Horizontal rule
          hr: () => (
            <hr className="my-4 border-0 h-0.5 bg-black" />
          ),

          // Strikethrough
          del: ({ children }) => (
            <del className="line-through text-gray-500">{children}</del>
          ),

          // Images
          img: ({ src, alt }) => (
            <img
              src={src}
              alt={alt || ''}
              className="brutal-border max-w-full my-2"
            />
          ),
        }}
      >
        {safeContent}
      </ReactMarkdown>
    </div>
  );
};
