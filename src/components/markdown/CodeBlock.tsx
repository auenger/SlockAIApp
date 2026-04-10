/**
 * CodeBlock — Shiki-powered syntax-highlighted code block component.
 *
 * Features:
 * - VSCode-level syntax highlighting via Shiki
 * - Language label display
 * - One-click copy to clipboard
 * - Line numbers (optional)
 * - Max height with scroll
 * - Neo-Brutalism styling
 */

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Copy, Check, Code } from 'lucide-react';
import { cn } from '../../lib/utils';

// Lazy-load Shiki to avoid blocking initial render
let shikiModule: typeof import('shiki') | null = null;
let shikiPromise: Promise<typeof import('shiki')> | null = null;

async function loadShiki() {
  if (shikiModule) return shikiModule;
  if (shikiPromise) return shikiPromise;
  shikiPromise = import('shiki').then((mod) => {
    shikiModule = mod;
    return mod;
  });
  return shikiPromise;
}

// Cache the highlighter instance
let highlighterInstance: import('shiki').Highlighter | null = null;
let highlighterPromise: Promise<import('shiki').Highlighter> | null = null;

async function getHighlighter(): Promise<import('shiki').Highlighter> {
  if (highlighterInstance) return highlighterInstance;
  if (highlighterPromise) return highlighterPromise;
  highlighterPromise = (async () => {
    const shiki = await loadShiki();
    const highlighter = await shiki.createHighlighter({
      themes: ['github-dark', 'github-light'],
      langs: [
        'javascript', 'typescript', 'python', 'rust', 'bash', 'json',
        'html', 'css', 'markdown', 'jsx', 'tsx', 'go', 'java', 'c',
        'cpp', 'yaml', 'toml', 'sql', 'shell', 'diff',
      ],
    });
    highlighterInstance = highlighter;
    return highlighter;
  })();
  return highlighterPromise;
}

// Resolve language aliases to Shiki-compatible names
function resolveLanguage(lang: string): string {
  const aliases: Record<string, string> = {
    js: 'javascript',
    ts: 'typescript',
    py: 'python',
    rs: 'rust',
    sh: 'bash',
    shell: 'bash',
    yml: 'yaml',
    md: 'markdown',
    text: 'markdown',
  };
  return aliases[lang.toLowerCase()] || lang.toLowerCase() || 'text';
}

interface CodeBlockProps {
  /** The raw code content */
  children: string;
  /** Language identifier (e.g. "typescript", "rust") */
  language?: string;
  /** Whether to show line numbers */
  showLineNumbers?: boolean;
  /** Optional className for the wrapper */
  className?: string;
}

export const CodeBlock: React.FC<CodeBlockProps> = ({
  children,
  language = '',
  showLineNumbers = true,
  className,
}) => {
  const [highlightedHtml, setHighlightedHtml] = useState<string>('');
  const [copied, setCopied] = useState(false);
  const [loading, setLoading] = useState(true);
  const codeRef = useRef<HTMLDivElement>(null);
  const resolvedLang = resolveLanguage(language);
  const displayLang = language || 'text';

  // Highlight the code using Shiki
  useEffect(() => {
    let cancelled = false;
    getHighlighter()
      .then((highlighter) => {
        if (cancelled) return;
        try {
          const html = highlighter.codeToHtml(children, {
            lang: resolvedLang,
            themes: {
              dark: 'github-dark',
              light: 'github-light',
            },
          });
          setHighlightedHtml(html);
        } catch {
          // Fallback: if the language is not supported, use plain text
          try {
            const html = highlighter.codeToHtml(children, {
              lang: 'text',
              themes: {
                dark: 'github-dark',
                light: 'github-light',
              },
            });
            setHighlightedHtml(html);
          } catch {
            // Ultimate fallback: just show plain text
            setHighlightedHtml('');
          }
        }
      })
      .catch(() => {
        // Shiki failed to load
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => { cancelled = true; };
  }, [children, resolvedLang]);

  // Copy to clipboard
  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(children);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard API not available
    }
  }, [children]);

  const lines = children.split('\n');
  // Remove trailing empty line from code blocks
  if (lines.length > 1 && lines[lines.length - 1] === '') {
    lines.pop();
  }
  const lineCount = lines.length;

  return (
    <div className={cn("brutal-border bg-white my-3 overflow-hidden", className)}>
      {/* Header bar */}
      <div className="flex items-center justify-between px-3 py-1.5 bg-gray-100 brutal-border-b">
        <div className="flex items-center gap-1.5">
          <Code size={12} className="text-gray-500" />
          <span className="text-[10px] font-black uppercase text-gray-600">
            {displayLang}
          </span>
          {lineCount > 1 && (
            <span className="text-[9px] text-gray-400">
              {lineCount} lines
            </span>
          )}
        </div>
        <button
          onClick={handleCopy}
          className={cn(
            "flex items-center gap-1 px-1.5 py-0.5 brutal-border text-[9px] font-bold transition-all",
            copied
              ? "bg-brutal-green text-black"
              : "bg-white hover:bg-gray-100 text-gray-600"
          )}
        >
          {copied ? (
            <>
              <Check size={10} />
              Copied
            </>
          ) : (
            <>
              <Copy size={10} />
              Copy
            </>
          )}
        </button>
      </div>

      {/* Code content */}
      <div className="relative overflow-auto max-h-[500px]">
        {loading ? (
          // Loading state: plain monospace text
          <pre className="p-3 text-xs leading-relaxed font-mono overflow-x-auto">
            {showLineNumbers ? (
              <table className="w-full border-collapse">
                <tbody>
                  {lines.map((line, i) => (
                    <tr key={i}>
                      <td className="pr-3 text-right text-gray-400 select-none w-8 align-top text-[10px]">
                        {i + 1}
                      </td>
                      <td className="align-top">
                        <code>{line || ' '}</code>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            ) : (
              <code>{children}</code>
            )}
          </pre>
        ) : highlightedHtml ? (
          // Shiki highlighted code
          <div className="relative">
            {showLineNumbers && (
              <div className="absolute left-0 top-0 bottom-0 w-8 bg-gray-50 brutal-border-r flex flex-col items-end pt-3 pr-2 select-none overflow-hidden">
                {Array.from({ length: lineCount }, (_, i) => (
                  <div key={i} className="text-[10px] text-gray-400 leading-[1.625]">
                    {i + 1}
                  </div>
                ))}
              </div>
            )}
            <div
              ref={codeRef}
              className={cn(
                "p-3 text-xs leading-relaxed overflow-x-auto shiki-code",
                showLineNumbers && "pl-11"
              )}
              dangerouslySetInnerHTML={{ __html: highlightedHtml }}
            />
          </div>
        ) : (
          // Fallback plain text
          <pre className="p-3 text-xs leading-relaxed font-mono overflow-x-auto">
            <code>{children}</code>
          </pre>
        )}
      </div>
    </div>
  );
};

/**
 * InlineCode — for inline code rendering within Markdown.
 */
export const InlineCode: React.FC<{ children: React.ReactNode; className?: string }> = ({
  children,
  className,
}) => {
  return (
    <code
      className={cn(
        "px-1.5 py-0.5 bg-gray-100 brutal-border text-xs font-mono",
        className
      )}
    >
      {children}
    </code>
  );
};
